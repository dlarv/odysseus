Turn a markdown style list of requirements into a spreadsheet.

ody [options] list-file [spreadsheet-file]

## Requirement Manager
Takes a markdown style list of requirements and translates them into a spreadsheet.

### Categories
Requirements can be organized into categories. All items in the list file that are not list items are treated as category declarations. If an abbreviated version is provided, this is used instead (see the example below).

```
These two categories are equivalent. 
    Category1 (Cat1)

    Cat1
```

All list items underneath a category declaration are added to that category.

### The List File
All list items are treated as requirements. Requirements contain the following info:
- category: See previous section.
- hash: See next section.
- contents: The body of the list item.
- status: Whether this item has been completed or not.
- id: A string of the form x.x.x. 

```
CAT1
1. These are the contents of the item w/ id=1.
    1. These are the contents of item w/ id=1.1.
    2. Item w/ id=1.2.
```

If the whitespace before the list header is greater than that of the previous line, '.1' is appended to the id. 
If the whitespace is less, the last .x of the id is dropped.
Otherwise, the last .x is incremented.

NOTE: The value of the id does not take the list item number into account. Furthermore, the exact amount of whitespace doesn't matter.
```
    1. This id = 1.
        2. This id = 1.1, not 1.2.
    2. This id = 2.
      1. This id = 2.1.
     3. This id = 3.
```
However, mixing tabs and spaces is discouraged, as this can lead to unexpected behavior.

Odysseus supports 3 types of lists: ordered, unordered, and todo.
```
Unordered lists:
- Item
+ Item
* Item

Ordered lists:
1. Item
a. Item
A. Item
The '.' is required at the end of ordered list headers. While the numbers are unbounded, only a-z/A-Z can be used (i.e. 'a.' is valid 'aa.' is not).

Todo lists:
- [ ] Item (status=0)
- [x] Item (status=1)
Any single character can be placed in the '[]', which is used to determine the item's status. ' ' and 'x' are special cases, being interpreted as 0 and 1 respectively. All other characters are interpreted as their respective ascii values.

Hybrid lists (combination of ordered and todo):
1. [ ]
```
### Spreadsheet File
The spreadsheet file is a csv file with the following columns:
    ```Hash,Category,Id,Contents,Status,Objective```

These largely line up with the Requirement fields in the previous section, other than Objective, which becomes relevant in the project manager mode.

The hash value is used as a unique identifier for each requirement and is used to connect an item between the list and spreadsheet files. If no value is provided in the list file, a new hash is generated using the contents field. However, a hash value can be provided using the (@hash) syntax.

```
- This item will generate a new hash.
- This item will use "hash" as its hash.(@hash)
```

Once both files have been loaded, odysseus will check to see if an item exists in both using this hash.

| List File | Spreadsheet | Result |
|-----------|-------------|--------|
| Exists    | Exists      | Status and Goal data are copied from spreadsheet. Rest of data is taken from list. |
| Exists    | !Exists     | New spreadsheet entry is created. |
| !Exists   | Exists      | Item is deleted. |

This means that the spreadsheet is used as an authority on a requirement's status and goal, while the list is the authority on everything else. This means that the id, contents, category, and hash are always drawn from the list.

When overwriting the txt file, if the csv provided a non-zero status, it will be saved as a hybrid list. Otherwise, it will be ordered.

## Project Manager
Though a few commands are exposed on the command line, this mode is primarily intended to be used via the tui. This tui can be accessed by using the -pT option, or just -p to access the cli.

### Project Registry
A list of all projects managed by odysseus is kept at $MYTHOS_LOCAL_DATA_DIR/odysseus/projects.toml. There are 3 types of projects odysseus can manage:

- Active: These are projects that you are currently working on.
- Backburner: These are projects that have taken a backseat, but you may plan on returning to someday.
- Archive: These are projects that have either been abandoned or completed.

### Projects
All projects have an entry in $MYTHOS_LOCAL_DATA_DIR/odysseus/\<project-name>.toml.

This file defines a working directory. When handling objectives (see below), odysseus will check for $working_dir/requirements.csv (which can be created using odysseus).  

Below is a complete list of the fields defined in this project file:
- Working directory
- Description
- Version
- Completed objectives

### Objectives
Each requirement can have an objective. This should not be confused with the requirement's id, though they share the same form. The objective refers to the version of the project. Once all requirements that share the same objective are completed, the project's version number is updated.

E.g. if requirements 1.1, 1.2, and 1.3 all share the same objective 1.0.0, once all 3 requirements are marked as finished, the project is now in version 1.0.0.

## Project Mode
#! TODO

Using ody -p gives access to project mode, which allows the user to manage their projects. It has two parts: the registry and project dashboards. The registry can be manipulated using either the cli or tui, while the latter can only be accessed thru the tui.

Registry Actions:
- List projects
- Add remove projects
- Rename project
- Add existing project
- Move project

Dashboard Actions:
- Show list of requirements.
- Show requirements sorted by objectives.
- Mark a requirement's status.
- View progress of project as a whole.
- View progress of each requirement.
- Add a new objective.
- Remove an objective.
- Assign/unassign a requirement to an objective.
