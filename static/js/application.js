const ROOM = "room";
const NAME = "name";
const SETTINGS = "settings";
const earthdawnStepActionDice = {
    1: '1d4-2',
    2: '1d4-1',
    3: '1d4',
    4: '1d6',
    5: '1d8',
    6: '1d10',
    7: '1d12',
    8: '2d6',
    9: '1d8+1d6',
    10: '1d10+1d6',
    11: '1d10+1d8',
    12: '2d10',
    13: '1d12+1d10',
    14: '2d12',
    15: '1d20+1d6',
    16: '1d20+1d8',
    17: '1d20+1d10',
    18: '1d20+1d12',
    19: '1d20+2d6',
    20: '1d20+1d8+1d6',
    21: '1d20+1d10+1d6',
    22: '1d20+1d10+1d8',
    23: '1d20+2d10',
    24: '1d20+1d12+1d10',
    25: '1d20+1d10+1d8+1d4',
    26: '1d20+1d10+1d8+1d6',
    27: '1d20+1d10+2d8',
    28: '1d20+2d10+1d8',
    29: '1d20+1d12+1d10+1d8',
    30: '1d20+1d10+1d8+2d6'
};

class myWebsocketHandler {

    setupSocket() {
        console.log("initializing connection")

        const wsUri =
            (window.location.protocol === 'https:' ? 'wss://' : 'ws://') +
            window.location.host +
            '/ws/'

        this.socket = new WebSocket(wsUri)

        this.socket.addEventListener("open", () => {
            const main = document.getElementById("chat-view")
            // main.innerText = ""
            main.classList.remove("disconnected")
            app.connected = true
            this.autoJoinMessages()
        })

        this.socket.addEventListener("message", (event) => {
            console.log(event)

            const eventContent = JSON.parse(event.data)
            if (eventContent.TextMessage)
                this.handleTextMessage(eventContent.TextMessage)

            if (eventContent.RoomState)
                handleRoomStateChange(eventContent.RoomState)
        })

        this.socket.addEventListener("close", () => {
            document.getElementById("chat-view").classList.add("disconnected")
            app.connected = false
            this.setupSocket()
        })
    }

    handleTextMessage(eventContent) {
        const isSystemMessage = eventContent.name == null;

        const pTag = document.createElement("div");
        pTag.className = "chatEntry";
        if (isSystemMessage)
            pTag.className += " systemMessage";
        const namePart = document.createElement("div");
        namePart.className = "name";
        if (!isSystemMessage)
            namePart.innerHTML = eventContent.name + ":";
        pTag.append(namePart);

        const messagePart = document.createElement("div");
        messagePart.className = "messagePart";
        pTag.append(messagePart);
        const request = document.createElement("div");
        request.className = "request";
        messagePart.append(request);
        const message = document.createElement("div");
        message.className = "message";
        messagePart.append(message);

        if (eventContent.dice_results == null) {
            message.innerHTML = eventContent.message;
        } else {
            message.innerHTML = eventContent.dice_results.join(" + ") + " = " + eventContent.dice_results.reduce((a, b) => a + b, 0);
            request.innerHTML = eventContent.message;
        }

        const time = document.createElement("div");
        time.className = "time";
        time.innerHTML = this.timeFromTimestamp(eventContent.time);
        pTag.append(time);

        document.getElementById("main").prepend(pTag);

        if (isSystemMessage)
            this.updateURLSearchParameters(eventContent.message)
        else
            updateInitiatives(eventContent)
    }

    submit(message) {
        this.socket.send(message)
    }

    autoJoinMessages() {
        const urlParams = new URLSearchParams(window.location.search)

        if (urlParams.has(NAME)) {
            let name = urlParams.get(NAME)
            this.submit("/name " + name)
        }

        if (urlParams.has(ROOM)) {
            let room = urlParams.get(ROOM)
            this.submit("/join " + room)
        }
    }

    updateURLSearchParameters(message) {
        const JOIN_MESSAGE_PREXIFX = "You joined room ";
        const NAME_MESSAGE_PREFIX = "You are now known as: "

        if (message.startsWith(JOIN_MESSAGE_PREXIFX)) {
            let roomName = message.split(JOIN_MESSAGE_PREXIFX)[1]
            updateURLSearchParameter("room", roomName)
        }

        if (message.startsWith(NAME_MESSAGE_PREFIX)) {
            let userName = message.split(NAME_MESSAGE_PREFIX)[1]
            updateURLSearchParameter("name", userName)
        }
    }

    timeFromTimestamp(timestamp) {
        let date = new Date(timestamp)
        // Hours part from the timestamp
        var hours = date.getHours();
        // Minutes part from the timestamp
        var minutes = "0" + date.getMinutes();
        // Seconds part from the timestamp
        var seconds = "0" + date.getSeconds();

        // Will display time in 10:30:23 format
        var formattedTime = hours + ':' + minutes.substr(-2) + ':' + seconds.substr(-2);
        return formattedTime
    }
}

function updateURLSearchParameter(key, value) {
    const url = new URL(window.location)
    const urlParams = new URLSearchParams(url.search)
    urlParams.set(key, value)
    url.search = urlParams
    history.replaceState({}, null, url)
}

function loadSettings() {
    const urlParams = new URLSearchParams(window.location.search)

    if (urlParams.has(SETTINGS)) {
        let base64Settings = urlParams.get(SETTINGS)
        let serializedSettings = atob(base64Settings)
        let settings = JSON.parse(serializedSettings)
        app.messageTemplates = settings.messageTemplates
        app.games.earthdawn.myKarma = settings.myKarma
    }
}

const iniRegex = /!.*\(ini(:.*)?\)(.*)/
function updateInitiatives(eventContent) {
    if (eventContent.message == "(clear initiative)")
        app.initiativeRolls = []

    let matches = eventContent.message.match(iniRegex)
    if (matches != null) {
        let description = matches.pop().trim()
        let mainName = eventContent.name
        let subName = matches.pop()
        if (subName)
            subName = subName.substr(1)

        let rolls = app.initiativeRolls.filter(i => !(i.mainName == mainName && i.subName == subName))
        rolls.push({
            result: eventContent.dice_results.reduce((a, b) => a + b, 0),
            description: description,
            mainName: mainName,
            subName: subName
        })
        rolls.sort((a, b) => b.result - a.result)
        app.initiativeRolls = rolls
    }
}

const diceLevelRegex = /!{1,2}\[(.*)\].*/
function expandStepLevel(message) {
    let matches = message.match(diceLevelRegex)
    if (matches != null) {
        let diceLevel = matches[1]
        let expanded = earthdawnStepActionDice[eval(diceLevel)]
        return message.replace("[" + diceLevel + "]", expanded)
    } else {
        return message
    }
}

function expandHideRoll(message) {
    let parts = message.split(' ')
    parts[0] = parts[0] + "*"
    return parts.join(' ')
}

function addKarma(message, karma) {
    let parts = message.split(' ')
    parts[0] = parts[0] + "+" + karma
    return parts.join(' ')
}

function handleRoomStateChange(eventContent) {
    app.room.name = eventContent.room_name
    eventContent.members.sort()
    app.room.members = eventContent.members
}

var app = new Vue({
    el: '#app',
    data: {
        currentText: '',
        useKarma: false,
        hideRoll: false,
        messageTemplates: [],
        edit: false,
        dragging: {
            templates: {
                hoverIndex: null
            }
        },
        toggleButton: {
            text: 'Edit'
        },
        visibilityToggles: {
            dice: true,
            initiative: true,
            templates: true
        },
        initiativeRolls: [],
        connected: false,
        room: {
            name: null,
            members: []
        },
        games: {
            earthdawn: {
                stepActionDice: earthdawnStepActionDice,
                myKarma: ''
            }
        }
    },
    methods: {
        toggleEdit: function () {
            this.edit = !this.edit
            this.toggleButton.text = this.edit ? "Done Editing" : "Edit"

            if (!this.edit)
                this.storeSettings()
        },
        storeSettings: function () {
            let serializedSettings = JSON.stringify({ messageTemplates: this.messageTemplates, myKarma: this.games.earthdawn.myKarma })
            let base64Settings = btoa(serializedSettings)
            updateURLSearchParameter("settings", base64Settings)
        },
        putToInputText: function (text) {
            this.currentText = text
            const input = document.getElementById("message")
            input.focus()
        },
        sendText: function (text) {
            this.putToInputText(text)
            this.submit()
        },
        submit: function (text) {
            var message
            if (text == null) {
                message = this.currentText
                this.currentText = ""
                message = expandStepLevel(message)

                if (this.useKarma) {
                    message = addKarma(message, this.games.earthdawn.myKarma)
                    this.useKarma = false
                }

                if (this.hideRoll) {
                    message = expandHideRoll(message)
                    this.hideRoll = false
                }
            } else {
                message = text
            }

            websocketClass.submit(message)
        },
        startDrag: function (evt, index) {
            evt.dataTransfer.dropEffect = 'move'
            evt.dataTransfer.effectAllowed = 'move'
            evt.dataTransfer.setData('itemIndex', index)
        },
        onDrop: function (evt, newIndex) {
            const oldIndex = evt.dataTransfer.getData('itemIndex')
            if (oldIndex == newIndex)
                return

            const item = this.messageTemplates[oldIndex]
            this.messageTemplates.splice(oldIndex, 1)
            if (newIndex > oldIndex)
                newIndex -= 1
            this.messageTemplates.splice(newIndex, 0, item)
        },
        endDrag: function(_evt) {
            this.dragging.templates.hoverIndex = null
        }
    }
})

websocketClass = new myWebsocketHandler()
websocketClass.setupSocket()

loadSettings()
