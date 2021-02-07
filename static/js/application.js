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
            })

            this.socket.addEventListener("message", (event) => {
                console.log(event)

                const eventContent = JSON.parse(event.data)
                const isSystemMessage = eventContent.name == null

                const pTag = document.createElement("div")
                pTag.className = "chatEntry"
                if (isSystemMessage)
                    pTag.className += " systemMessage"
                const namePart = document.createElement("div")
                namePart.className = "name"
                if (!isSystemMessage)
                    namePart.innerHTML = eventContent.name + ":"
                pTag.append(namePart)

                const messagePart = document.createElement("div")
                messagePart.className = "messagePart"
                pTag.append(messagePart)
                const request = document.createElement("div")
                request.className = "request"
                messagePart.append(request)
                const message = document.createElement("div")
                message.className = "message"
                messagePart.append(message)

                if (eventContent.dice_results == null) {
                    message.innerHTML = eventContent.message
                } else {
                    message.innerHTML = eventContent.dice_results.join(" + ") + " = " + eventContent.dice_results.reduce((a, b) => a + b, 0)
                    request.innerHTML = eventContent.message
                }

                document.getElementById("main").prepend(pTag)
            })

            this.socket.addEventListener("close", () => {
                document.getElementById("chat-view").classList.add("disconnected")
                this.setupSocket()
            })
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
    }

    websocketClass = new myWebsocketHandler()
    websocketClass.setupSocket()

    document.getElementById("button_chat")
        .addEventListener("click", (event) => {
            event.preventDefault();
            websocketClass.submit();
        })
})()