const ROOM = "room";
const NAME = "name";
const SETTINGS = "settings";

(() => {
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

                this.autoJoinMessages()
            })

            this.socket.addEventListener("message", (event) => {
                console.log(event)

                const eventContent = JSON.parse(event.data)
                if (eventContent.TextMessage)
                    this.handleTextMessage(eventContent.TextMessage)
            })

            this.socket.addEventListener("close", () => {
                document.getElementById("chat-view").classList.add("disconnected")
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

        submit(msg) {
            var message
            if (msg == null) {
                const input = document.getElementById("message")
                message = input.value
                input.value = ""
            } else {
                message = msg
            }

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

    websocketClass = new myWebsocketHandler()
    websocketClass.setupSocket()

    document.getElementById("button_chat")
        .addEventListener("click", (event) => {
            event.preventDefault();
            websocketClass.submit();
        })
})()

function updateURLSearchParameter(key, value) {
    const url = new URL(window.location)
    const urlParams = new URLSearchParams(url.search)
    urlParams.set(key, value)
    url.search = urlParams
    history.replaceState({}, null, url)
}

function loadTemplates() {
    const urlParams = new URLSearchParams(window.location.search)

    if (urlParams.has(SETTINGS)) {
        let base64Settings = urlParams.get(SETTINGS)
        let serializedSettings = atob(base64Settings)
        let settings = JSON.parse(serializedSettings)
        return settings.messageTemplates
    } else {
        return []
    }
}

const iniRegex = /!.*\(ini\)(.*)/
function updateInitiatives(eventContent) {
    if (eventContent.message == "(clear initiative)")
        app.initiativeRolls = []
    
    let matches = eventContent.message.match(iniRegex)
    if (matches != null) {
        let description = matches.pop().trim()
        let rolls = app.initiativeRolls.filter(i => i.name != eventContent.name)
        rolls.push({
            result: eventContent.dice_results.reduce((a, b) => a + b, 0),
            description: description,
            name: eventContent.name
        })
        rolls.sort((a,b) => b.result-a.result)
        app.initiativeRolls = rolls
    }
}

var app = new Vue({
    el: '#app',
    data: {
        message: 'Hello Vue!',
        messageTemplates: loadTemplates(),
        edit: false,
        toggleButton: {
            text: 'Edit'
        },
        initiativeRolls: []
    },
    methods: {
        toggleEdit: function () {
            app.edit = !app.edit
            app.toggleButton.text = app.edit ? "Done Editing" : "Edit"

            // TODO if now edit==false, store the message templates in the url
            let serializedSettings = JSON.stringify({ messageTemplates: app.messageTemplates })
            let base64Settings = btoa(serializedSettings)
            updateURLSearchParameter("settings", base64Settings)
        },
        putToInputText: function (template) {
            const input = document.getElementById("message")
            input.value = template.text
            input.focus()
        },
        executeTemplate: function (template) {
            this.putToInputText(template)
            const submitButton = document.getElementById("button_chat")
            submitButton.click()
        }
    }
})
