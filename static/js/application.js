const ROOM = "room";
const NAME = "name";

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