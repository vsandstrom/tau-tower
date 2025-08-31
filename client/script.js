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


