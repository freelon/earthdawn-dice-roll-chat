# Earthdawn Dice Rolling Chat

Kabuja!

This is based on the [actix-websocket-chat example](https://github.com/actix/examples/tree/master/websocket-chat).

## Changelog

### 0.2

* Dice rolls accept appended text, separated by a space (i.e. `!!3d6 Initiative` rolls 3d6 but keeps the text 'Initiative')
* Dice specification defaults to one dice (`!1d6` can be written as `!d6`)
