# TODO

  - debug logging
    - add debug logging to other components (files) also
    - add --debug -d option that enables debug output, and disable it by default
  - improve error handling (os and other libraries could raise exceptions)
  - check return values and output from git commands (currently only status output is checked)
  - implement create_local_config() and create_project_config()
  - commands
    - initialize or init
      - create templates for new configuration files, if missing
      - show different output if everything is missing,
        only local config is missing, only some project is missing, etc
      - support --force, with confirmation, to recreate all projects (rm & clone)
    - status
      - also check that source is on master branch?
      - a separate command or option to list projects and branches they are on
        - or even do that by default?
    - update or pull
      - possibility to specify branch? (by defaut master)
      - give an error if
        - project is not on the specified branch
        - project has changes
        - branch doesn't exist
        - git command fails
    - project add
      - abort on configuration error (suggest init)
        - abort if server alias doens't exist
      - add new subprojects to config
      - add new subprojects to .gitignore automatically
      - only write config file after successful cloning
    - server add
      - abort if configuration error (suggest init)
      - new server alias, don't allow duplicates

