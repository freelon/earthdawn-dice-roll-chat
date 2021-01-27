(() => {
    class myWebsocketHandler {
        setupSocket() {

            const wsUri =
            (window.location.protocol === 'https:' ? 'wss://' : 'ws://') +
            window.location.host +
            '/ws/'

            this.socket = new WebSocket(wsUri)

            this.socket.addEventListener("open", () => {
                const main = document.getElementById("main")
                main.innerText = ""
                main.classList.remove("disconnected")
            })

            this.socket.addEventListener("message", (event) => {
                console.log(event)

                const eventContent = JSON.parse(event.data)

                const pTag = document.createElement("div")
                pTag.className = "chatEntry"
                const namePart = document.createElement("div")
                namePart.className = "name"
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

                if (eventContent.diceResults == null) {
                    message.innerHTML = eventContent.message
                } else {
                    message.innerHTML = eventContent.diceResults.join(" + ") + " = " + eventContent.diceResults.reduce((a, b) => a + b, 0)
                    request.innerHTML = eventContent.message
                }

                document.getElementById("main").prepend(pTag)
            })

            this.socket.addEventListener("close", () => {
                document.getElementById("main").classList.add("disconnected")
                this.setupSocket()
            })
        }

        submit(event, msg) {
            event.preventDefault()
            var message
            if (msg == null) {
                const input = document.getElementById("message")
                message = input.value
                input.value = ""
            } else {
                message = msg
            }

            this.socket.send(
                // JSON.stringify({
                //     data: {
                //         message: message,
                //         name: name
                //     },
                // })
                message
            )
        }
    }

    const websocketClass = new myWebsocketHandler()
    websocketClass.setupSocket()

    document.getElementById("button_chat")
        .addEventListener("click", (event) => websocketClass.submit(event))
})()