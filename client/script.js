let c = 0;

const decoder = new window["ogg-opus-decoder"].OggOpusDecoderWebWorker({
  forceStereo: false,
  speechQualityEnhancement: 'nolace',
  sampleRate: 48000
});

decoder.ready.then(() => {
  console.log("hello")
});

for (let i = 0; i < 100; i++) {
   console.log(i)
}


var ws = new WebSocket("ws://127.0.0.1:9001");
ws.binaryType = "arraybuffer";

ws.addEventListener("open", () => {
  console.log("hello")
});

ws.addEventListener('connection', socket => {
  socket.send('Hello from server!')
});

ws.addEventListener('message', msg => {
  console.log(msg.data)
  console.log(c++);
})


