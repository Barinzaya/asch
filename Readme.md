# Mark AI Traffic

## What is it?

This is a very lightweight utility intended to be used with [AssettoServer](https://github.com/compujuckel/AssettoServer), to add the necessary key for cars that are intended for use as traffic.

## Why?

Because every time the entry list is re-packed via Content Manager, the entry list has to be manually edited to re-add the key to cars that are intended for use as traffic. This tool allows the info that CM *does* support (the entry's team name) to be used to indicate which cars should be used as traffic.

## How?

When run, it will look for the entry list (in `cfg/entry_list.ini`), read its contents, and look for any cars that meet any of the following criteria:

- The car's internal name contains the word "traffic" (most likely, it is from [the traffic pack](https://drive.google.com/file/d/1PbYGD61LFi0DNcw7f2o68xBwdJ80pZfl/view)). It will be set to `AI=fixed`.
- The entry's team includes "traffic". It will be set to `AI=fixed`.
- The entry's team includes "reserve". It will be set to `AI=auto`.

IF any cars meeting these criteria are found, the updated entry list will be written out, replacing the old one, and the server will be ready to run.
