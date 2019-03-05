#! /usr/bin/env python3

import sys
import logging
import subprocess
import yaml
import os
import getopt

from config import Config
from utils import RepoStatus, ProjectStatus

EXIT_SUCCESS = 0
EXIT_FAILURE = 1

class RepoCLI():
    """Repo command line user interface"""
    def __init__(self):
        self.command = None

    def show(self, message):
        """Show a message to the user"""
        logging.info(message)
        print(message)

    def show_status(self, statuses):
        """Show combined project statuses"""
        print("+" + "-" * 42 + "+" + "-" * 22 + "+")
        print("| {:<40} | {:<20} |".format("Project", "Status"))
        print("+" + "-" * 42 + "+" + "-" * 22 + "+")
        for status in statuses:
            print("| {:<40} | {:<20} |".format(status[0], status[1].name))
        print("+" + "-" * 42 + "+" + "-" * 22 + "+")

    def usage(self):
        """List all available commands and options to the user"""
        print("Commands:")
        print("  help                   show this help")
        print("  status                 get combined repository status")
        print("  init                   initialize database")
        print("  update                 update all repositories")
        print("")

    def parse_options(self, argv):
        """Parse command line options."""

        all_commands = ["status",
                        "init",
                        "update"]
        short_options = "hd"
        long_options = ["help", "debug"]

        logging.info('Parsing command line options')

        try:
            opts, args = getopt.getopt(argv, short_options, long_options)
        except getopt.error as msg:
            self.usage()
            return RepoStatus.GETOPT_ERROR

        # process options
        for opt, _ in opts:
            if opt in ("-h", "help"):
                self.usage()
            if opt in ("-d", "debug"):
                logging.getLogger().setLevel(logging.DEBUG)
            else:
                logging.warning("Unknown option: {}", opt)
                self.usage()
                return RepoStatus.INVALID_ARGUMENTS

        # process command line arguments here
        if not args:
            print("No command issued.")
            self.usage()
            return RepoStatus.INVALID_ARGUMENTS

        if len(args) != 1:
            return RepoStatus.INVALID_ARGUMENTS

        if args[0] in all_commands:
            self.command = args[0]
        else:
            print("Invalid command issued.")
            return RepoStatus.INVALID_ARGUMENTS

        # parsing options and arguments succeeded
        logging.debug("Command requested: {}".format(self.command))

        if self.command is None:
            self.usage()
            return RepoStatus.INVALID_ARGUMENTS

        return RepoStatus.OK

class Repo():
    """Git workarea management tool"""
    def __init__(self, config, ui):
        self.config = config
        self.ui = ui

    def initialize(self):
        """
        Initialize the project environment

        Check if configuration files exist, and create them, if needed.
        Projects or servers are not added. User needs to add them manually, or
        using separate commands.

        Clone any missing projects.

        This can be safely executed on an environment that has been already initialized.
        """
        logging.info("Initializing repository")

        if self.config.project_config_exists() == False:
            self.ui.show("Project configuration file doesn't exist - creating")
            status = self.config.create_project_config()
            if status != RepoStatus.OK:
                self.ui.show("Creating project configuration file failed")
                return status
        else:
            self.ui.show("Project configuration file exists")

        if self.config.local_config_exists() == False:
            self.ui.show("Local configuration file doesn't exist - creating")
            status = self.config.create_local_config()
            if status != RepoStatus.OK:
                self.ui.show("Creating local configuration file failed")
                return status
        else:
            self.ui.show("Local configuration file exists")

        if self.config.read_local_configuration() != RepoStatus.OK:
            return RepoStatus.CONFIGURATION_ERROR
        if self.config.read_project_configuration() != RepoStatus.OK:
            return RepoStatus.CONFIGURATION_ERROR

        for project in self.config.project_config.projects:
            if project["path"] in [".", ""]:
                logging.info("Skipping root project: '{}'".format(project["name"]))
                continue
            if os.path.exists(project["path"]):
                self.ui.show("Project '{}' already exists".format(project["name"]))
                continue
            status = self._clone_project(project)
            if status == RepoStatus.OK:
                return RepoStatus.OK
            if status == RepoStatus.CONFIGURATION_ERROR:
                self.ui.show("Cloning '{}' failed. Configuration error.".format(project["name"]))
                return RepoStatus.CONFIGURATION_ERROR
            else:
                self.ui.show("Error cloning project '{}'".format(project["name"]))
                return RepoStatus.GIT_ERROR

        return RepoStatus.OK

    def status(self):
        """Get composite git status of all repos"""
        statuses = []
        status = ProjectStatus.CLEAN
        for project in self.config.project_config.projects:
            status = self._check_single_project_status(project)
            statuses.append((project["name"], status))
        self.ui.show_status(statuses)
        return statuses

    def update(self):
        """Update (pull) all projects, including the top level repo and all subprojects"""
        #TODO: check preconditions, if status clean, correct branch, etc?
        for project in self.config.project_config.projects:
            self.ui.show("Updating project '{}'".format(project["name"]))
            status = self._pull_project(project)
            if status != RepoStatus.OK:
                self.ui.show("Updating project '{}' failed".format(project["name"]))
                return status
        return RepoStatus.OK

    def _get_git_url(self, project):
        git_server_alias = project["git_server_alias"]
        git_server = self.config.get_server_url_by_alias(git_server_alias)
        if git_server is None:
            logging.info("Configuration error: "
                    "Server url matching alias '{}' not found".format(git_server_alias))
            return RepoStatus.CONFIGURATION_ERROR
        url = git_server + project["git_path"]
        return url

    def _pull_project(self, project):
        """Update a single subproject"""
        cmd_status, _ = self._run_command_on_project(project, ["git", "pull"])
        return cmd_status
        #TODO: add some checks first (or do them in update())

    def _clone_project(self, project):
        """Clone a single subproject"""
        try:
            url = self._get_git_url(project)
        except KeyError as e:
            return RepoStatus.CONFIGURATION_ERROR

        command = ["git", "clone", url, project["path"]]
        logging.info("Cloning project '{}' from '{}'".format(project["name"], url))
        status, _ = self._run_command_on_project(project, command, False)

        return status

    def _check_single_project_status(self, project):
        """Check status of a single (sub)project"""
        status = ProjectStatus.UNKNOWN
        cmd_status, output = self._run_command_on_project(project, ["git", "status", "--porcelain"])

        if cmd_status == RepoStatus.OK:
            if output == b'':
                status = ProjectStatus.CLEAN
            else:
                status = ProjectStatus.CHANGES
        elif cmd_status == RepoStatus.OS_ERROR:
            status = ProjectStatus.UNINITIALIZED
        elif cmd_status == RepoStatus.COMMAND_ERROR:
            status = ProjectStatus.UNKNOWN

        logging.info("Project '{}' status: {}".format(project["name"], status))
        return status

    def _run_command_on_project(self, project, command, change_dir=True):
        """Run a single command on a project and get output"""
        output = None
        status = RepoStatus.OK

        try:
            cwd = os.getcwd()
            if change_dir:
                logging.info("Changing path to '{}'".format(self.config.project_path + project["path"]))
                os.chdir(self.config.project_path + project["path"])
            logging.info("Running subprocess '{}'".format(command))
            output = subprocess.check_output(command)
            logging.info("Changing path back to '{}'".format(cwd))
            os.chdir(cwd)
        except OSError as e:
            logging.info("Unable to access subproject: {}".format(e))
            status = RepoStatus.OS_ERROR
        except subprocess.CalledProcessError as e:
            logging.info("Subprocess returned error: {}".format(e.errorcode))
            logging.info("Subprocess command: {}".format(e.cmd))
            logging.info("Subprocess output: {}".format(e.output))
            status = RepoStatus.COMMAND_ERROR

        return status, output


def main(argv):
    """Main function of repo tool"""
    logging.basicConfig(stream=sys.stderr, level=logging.CRITICAL)

    config = Config()
    ui = RepoCLI()
    repo = Repo(config, ui)

    logging.info("Local configuration file exists: {}".format(config.local_config_exists()))
    logging.info("Project configuration file exists: {}".format(config.project_config_exists()))

    if repo.ui.parse_options(argv) != RepoStatus.OK:
        return EXIT_FAILURE

    if ((config.local_config_exists() == False) and
        (config.project_config_exists() == False) and
        (repo.ui.command != 'init')):
        print("Repo not initialized. Run 'repo init' to initialize")
        return EXIT_FAILURE

    if repo.ui.command == 'init':
        status = repo.initialize()
        if status == RepoStatus.OK:
            repo.ui.show("Done")
        else:
            repo.ui.show("Initialization failed")

    if config.read_local_configuration() != RepoStatus.OK:
        return EXIT_FAILURE
    if config.read_project_configuration() != RepoStatus.OK:
        return EXIT_FAILURE

    if repo.ui.command == 'status':
        repo.status()
    elif repo.ui.command == 'update':
        repo.update()
    else:
        return EXIT_FAILURE

if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
