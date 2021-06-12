# Earthdawn Dice Rolling Chat

Kabuja!

This is based on the [actix-websocket-chat example](https://github.com/actix/examples/tree/master/websocket-chat).

## Changelog

### 0.7

* Configurable port
* Welcome message including program version and build version (i.e. git commit the build is based on)

### 0.6

* Added karma setting and a checkbox to add karma to the current roll
* Added quick step input in roll (i.e. write '!![3]' for a step of 3 instead of typing '!!1d4' manually)

### 0.5

* Send the current room state to users (room name and all member names)

### 0.4

* Added initiative management

### 0.3

* Added message templates that are stored as a query parameter.

### 0.2

* The username and last entered room are appended to the URL as query parameters and will be used automatically upon connecting. After a connection loss, these are used to rejoin the previous room. They can also be used to create a bookmark, allowing a user to "save" its name and room.
* Dice rolls accept appended text, separated by a space (i.e. `!!3d6 Initiative` rolls 3d6 but keeps the text 'Initiative')
* Dice specification defaults to one dice (`!1d6` can be written as `!d6`)
