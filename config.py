#! /usr/bin/env python3

import os
import yaml
import logging

from utils import RepoStatus, ProjectStatus, find_file_along_path

class LocalConfigYaml(yaml.YAMLObject):

    yaml_tag = u'!repo.random.org,2019-03-02/config'

    def __init__(self, servers):
        self.servers = {}
        super(LocalConfigYaml, self).__init__()

    def __repr__(self):
        return "{} (servers={}".format(self.__class__.__name__, self.servers)

class ProjectConfigYaml(yaml.YAMLObject):

    yaml_tag = u'!repo.random.org,2019-03-02/projects'

    def __init__(self, servers):
        self.projects = {}
        super(ProjectConfigYaml, self).__init__()

    def __repr__(self):
        return "{} (projects={}".format(self.__class__.__name__, self.projects)

class Config():
    """Manage configuration file(s)"""
    def __init__(self, path="."):
        self.local_config_file = ".repo.yaml"
        self.project_config_file = "projects.yaml"
        try:
            self.project_path = find_file_along_path(self.local_config_file, path) + "/"
        except Exception:
            self.project_path = None

        self.local_config = None
        self.project_config = None

    def local_config_exists(self):
        """Check if local configuration file exists"""
        if self.project_path is None:
            return False
        return os.path.isfile(self.project_path + self.local_config_file)

    def project_config_exists(self):
        """
        Check if project configuration file exists

        If the file wasn't found during __init__(), then assume it doesn't exist.
        """
        return self.project_path is not None

    def create_local_config(self):
        """Create an new local configuration file"""
        servers = []
        self.local_config = LocalConfigYaml(servers)
        try:
            with open(self.local_config_file, 'w') as configuration_file:
                yaml_data = yaml.dump(self.local_config,
                                      default_flow_style=False,
                                      explicit_start=True)
                configuration_file.write(yaml_data)
        except Exception:
            logging.error("Error creating local configuration file")
            return RepoStatus.CONFIGURATION_ERROR
        return RepoStatus.OK

    def create_project_config(self):
        """Create an new project configuration file"""
        projects = []
        self.project_path = "./"
        self.project_config = ProjectConfigYaml(projects)
        try:
            with open(self.project_config_file, 'w') as configuration_file:
                yaml_data = yaml.dump(self.project_config,
                                      default_flow_style=False,
                                      explicit_start=True)
                configuration_file.write(yaml_data)
        except Exception:
            logging.error("Error creating project configuration file")
            return RepoStatus.CONFIGURATION_ERROR
        return RepoStatus.OK

    def read_local_configuration(self):
        """Read local configuration file"""
        try:
            logging.info("Loading local configuration from {}{}".format(self.project_path,
                                                                        self.local_config_file))
            with open(self.project_path + self.local_config_file) as configuration_file:
                try:
                    self.local_config = yaml.load(configuration_file)
                except Exception:
                    logging.info("Error reading local configuration file")
                    return RepoStatus.CONFIGURATION_ERROR

            logging.debug(self.local_config)
        except EnvironmentError:
            logging.error("Error reading local configuration file")
            return RepoStatus.CONFIGURATION_ERROR
        return RepoStatus.OK

    def read_project_configuration(self):
        """Read project configuration file"""
        logging.info("Loading project configuration from {}{}".format(self.project_path,
                                                                      self.project_config_file))
        try:
            with open(self.project_path + self.project_config_file) as configuration_file:
                self.project_config = yaml.load(configuration_file)

                logging.info(self.project_config)
        except EnvironmentError:
            logging.error("Error reading project configuration file")
            return RepoStatus.CONFIGURATION_ERROR
        return RepoStatus.OK

    def get_server_url_by_alias(self, alias):
        """Get Git server URL by alias from local configuration"""
        server_entry = next((x for x in self.local_config.servers if x["alias"] == alias), None)
        if server_entry is None:
            return None
        return server_entry["server"]
