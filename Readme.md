# AssettoServer Config Helper (ASCH)

## What is it?

This is a very lightweight utility intended to be used with [AssettoServer](https://github.com/compujuckel/AssettoServer), to apply some necessary config changes for running servers with AI traffic and/or at night-time. It can currently:

* Update the entry list to mark probable traffic vehicles as AI vehicles.
* Update the server configuration with a `SUN_ANGLE` value corresponding to a configurable time of day.

## How do I use it?

Put the executable in the same directory as your AssettoServer executable, and run it whenever your server config is updated. The utility will create a config file for itself in the `cfg` directory, which will allow configuration of what the utility will do. By default, this configuration will cause the utility to make no changes.

Make sure you set up your config in CM to identify which cars you want used as traffic; see below.

## Why?

Because every time the entry list is re-packed via Content Manager, the entry list has to be manually edited to re-add the key to cars that are intended for use as traffic, and the server configuration has to be updated to set the time of day if you wish to ues a time outside of the range supported by vanilla AC (8 AM to 6 PM). This tool allows the info that CM *does* support (the entry's team name) to be used to indicate which cars should be used as traffic, and to allow a time of day to be configured.

## AI Traffic Setup

If enabled in the configuration, ASCH will look for the entry list (in `cfg/entry_list.ini`), read its contents, and look for any cars that meet any of the following criteria:

- The car's internal name contains the word "traffic" (most likely, it is from [the traffic pack](https://drive.google.com/file/d/1PbYGD61LFi0DNcw7f2o68xBwdJ80pZfl/view)). It will be set to `AI=fixed`.
- The entry's team includes "traffic". It will be set to `AI=fixed`.
- The entry's team includes "reserve". It will be set to `AI=auto`.

IF any cars meeting these criteria are found, the updated entry list will be written out, replacing the old one, and the server will be ready to run.

## Time of Day

If a time is set in the configuration, ASCH will look for the server configuration (in `cfg/server_cfg.ini`) and update it to set the time of day to the configured time.

If no time is set, the server configuration will not be changed.
