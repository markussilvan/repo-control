#! /usr/bin/env python
# -*- coding: utf-8 -*-

import datetime
import yaml


class YamlConfig(object):
    """
    YAML library configuration/extension
    """
    def __init__(self):
        """
        Initialize YamlConfig instance
        """
        if isinstance(self, YamlConfig):
            raise Exception("YamlConfig should not be instantiated")

    @staticmethod
    def add_representers():
        """
        Add new representers for PyYaml library
        """
        yaml.add_representer(str, YamlConfig.represent_str)
        yaml.add_representer(datetime.datetime, YamlConfig.represent_datetime)

    @staticmethod
    def represent_str(dumper, data):
        tag = None
        if '\n' in data:
            style = '|'
        else: style = None
        data = bytes(data, 'utf-8')
        try:
            data = str(data, 'ascii')
            tag = u'tag:yaml.org,2002:str'
        except UnicodeDecodeError:
            try:
                data = str(data, 'utf-8')
                tag = u'tag:yaml.org,2002:str'
            except UnicodeDecodeError:
                data = data.encode('base64')
                tag = u'tag:yaml.org,2002:binary'
                style = '|'
        return dumper.represent_scalar(tag, data, style=style)

    @staticmethod
    def represent_datetime(dumper, data):
        value = str(data.isoformat(' ') + ' Z')
        return dumper.represent_scalar(u'tag:yaml.org,2002:timestamp', value)

