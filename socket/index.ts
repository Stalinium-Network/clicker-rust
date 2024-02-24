import { io } from "socket.io-client";

// Указываем правильный URL для подключения к серверу
const socket = io("http://127.0.0.1:3000/", {
    query: {
        id: "1",
        password: '123'
    },
});

// Отправляем сообщение сразу после установления подключения
socket.on('connect', () => {
    console.log("connected");
    socket.emit("msg", "hello"); // Правильное место для отправки сообщения
    console.log(socket.connected); // Теперь здесь будет true
});

// Исправлено на правильное событие для отслеживания отключения
socket.on('disconnect', () => {
    console.log("disconnected");
});

socket.on("error", (error) => {
    console.log("error", error); // Лучше логировать саму ошибку
});
