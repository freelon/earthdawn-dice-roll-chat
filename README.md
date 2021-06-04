# Earthdawn Dice Rolling Chat

Kabuja!

This is based on the [actix-websocket-chat example](https://github.com/actix/examples/tree/master/websocket-chat).

## Changelog

### 0.2

* The username and a default room can be provided as query parameters and will be used automatically upon connecting. After a connection loss, these are used to rejoin the previous room. They can also be used to create a bookmark, allowing a user to "save" its name and room.
* Dice rolls accept appended text, separated by a space (i.e. `!!3d6 Initiative` rolls 3d6 but keeps the text 'Initiative')
* Dice specification defaults to one dice (`!1d6` can be written as `!d6`)
