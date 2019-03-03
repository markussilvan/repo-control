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
        print(message)

    def show_status(self, statuses):
        """Show combined project statuses"""
        print("| {:<40} | {:<20} |".format("Project", "Status"))
        print("-" * (60 + 7))
        for status in statuses:
            print("| {:<40} | {:<20} |".format(status[0], status[1].name))

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
            logging.info("Project configuration file doesn't exist")
            self.ui.show("Project configuration file doesn't exist")
            self.config.create_project_config()
            #TODO: error handling
        else:
            logging.info("Project configuration file exists")
            self.ui.show("Project configuration file exists")

        if self.config.local_config_exists() == False:
            logging.info("Local configuration file doesn't exist")
            self.ui.show("Local configuration file doesn't exist")
            self.config.create_local_config()
            #TODO: error handling
        else:
            logging.info("Local configuration file exists")
            self.ui.show("Local configuration file exists")

        for project in self.config.project_config.projects:
            if os.path.exists(project["path"]):
                logging.info("Project {} already exists".format(project["name"]))
                self.ui.show("Project {} already exists".format(project["name"]))
                continue
            if self._clone_project(project) != RepoStatus.OK:
                logging.info("Error cloning project {}".format(project["name"]))
                self.ui.show("Error cloning project {}".format(project["name"]))
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
        pass

    def _clone_project(self, project):
        """Clome a single subproject"""
        try:
            git_server_alias = project["git_server_alias"]
            git_server = self.config.get_server_url_by_alias(git_server_alias)
            if git_server is None:
                logging.info("Configuration error: "
                             "Server url matching alias '{}' not found".format(git_server_alias))
                return RepoStatus.CONFIGURATION_ERROR
            git_path = project["git_path"]
            url = git_server + git_path
            logging.info("Cloning project '{}' from '{}'".format(project["name"], url))
            output = subprocess.check_output(["git", "clone", url, project["path"]])
            logging.info("Command output: {}".format(output))
        except subprocess.CalledProcessError as e:
            logging.info("Subprocess returned error: {}".format(e))
            return RepoStatus.GIT_ERROR
        return RepoStatus.OK

    def _check_single_project_status(self, project):
        """Check status of a single (sub)project"""
        output = None
        status = ProjectStatus.UNKNOWN

        try:
            cwd = os.getcwd()
            os.chdir(self.config.project_path + project["path"])
            output = subprocess.check_output(["git", "status", "--porcelain"])
            os.chdir(cwd)
        except OSError:
            status = ProjectStatus.UNINITIALIZED
        except subprocess.CalledProcessError:
            logging.info("Subprocess returned error: {}".format(e))
            status = ProjectStatus.UNKNOWN
        else:
            if output == b'':
                status = ProjectStatus.CLEAN
            else:
                status = ProjectStatus.CHANGES

        logging.info("Project '{}' status: {}".format(project["name"], status))
        return status


def main(argv):
    """Main function of repo tool"""
    logging.basicConfig(stream=sys.stderr, level=logging.CRITICAL)

    config = Config()
    ui = RepoCLI()
    repo = Repo(config, ui)

    logging.info("Local configuration file exists: {}".format(config.local_config_exists()))
    logging.info("Project configuration file exists: {}".format(config.project_config_exists()))

    if (config.local_config_exists() and config.project_config_exists()) == False:
        print("Repo not initialized. Run 'repo init' to initialize")
        return EXIT_FAILURE

    if config.read_local_configuration() != RepoStatus.OK:
        return EXIT_FAILURE
    if config.read_project_configuration() != RepoStatus.OK:
        return EXIT_FAILURE

    if repo.ui.parse_options(argv) != RepoStatus.OK:
        return EXIT_FAILURE

    if repo.ui.command == 'init':
        status = repo.initialize()
        if status == RepoStatus.OK:
            repo.ui.show("Done")
        else:
            repo.ui.show("Initialization failed")
    elif repo.ui.command == 'status':
        repo.status()
    else:
        return EXIT_FAILURE

if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
