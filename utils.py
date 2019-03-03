#! /usr/bin/env python3
# -*- coding: utf-8 -*-

"""
General utils for Repo
"""

import os
import stat
import shutil
from enum import Enum

class ProjectStatus(Enum):
    CLEAN = 0
    CHANGES = 1
    UNINITIALIZED = 2
    UNKNOWN = 3

class RepoStatus(Enum):
    OK = 0
    INVALID_ARGUMENTS = 1
    GETOPT_ERROR = 2
    GIT_ERROR = 3
    CONFIGURATION_ERROR = 4

def find_file_along_path(filename, path="."):
    """
    Find a file from the given path.
    File is searched from every directory towards the root
    until it is found or a device boundary is reached.

    os.stat doesn't work on Windows before Python 3.4.
    Nevertheless, a root of a drive is reached at some point
    and the loop will end.

    Parameters:
    - filename: file to search
    - path: (optional) path to start the search from.
            Defaults to current directory.

    Returns:
    - path to the file not including the file name

    Raises:
    - Exception: root or device boundary reached before file is found
    """
    path = os.path.realpath(path)
    s = os.stat(path)[stat.ST_DEV]
    ps = s

    while path != '/' and path[1:] != ":\\":
        parent = os.path.dirname(path)
        ps = os.stat(parent)[stat.ST_DEV]

        # check if this directory contains dit config
        if os.path.isfile("{}/{}".format(path, filename)):
            return str(path)

        if ps == s:
            path = parent
        else:
            # not the same device anymore
            raise Exception("Can't find file on this device")


    # root dir
    if os.path.isfile("{}/{}".format(path, filename)):
        return str(path)

    raise Exception("Can't find the file")
