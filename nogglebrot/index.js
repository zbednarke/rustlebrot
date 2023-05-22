const fs = require('fs');
const PNG = require('pngjs').PNG;
const { exec } = require('child_process');
const cluster = require('cluster');
const numCPUs = require('os').cpus().length;

function sinebowColor(iterRatio) {
    // 0 <= i < 1= 1
    let pi2 = Math.PI * 2;

    let r = Math.sin(pi2 * iterRatio) * 255;
    let g = Math.sin(pi2 * iterRatio + 2 * Math.PI / 3) * 255;
    let b = Math.sin(pi2 * iterRatio + 4 * Math.PI / 3) * 255;

    return { r: Math.floor(r), g: Math.floor(g), b: Math.floor(b) };
}

function mandelbrot(c, maxIter) {
    // Returns 1 if the point never escaped, else the number of iters to escape / maxIter
    // aka iterRatio
    let z = { x: 0, y: 0 };
    let n = 0;
    while (n < maxIter) {
        let zz = { x: z.x * z.x - z.y * z.y, y: 2 * z.x * z.y };
        z = { x: zz.x + c.x, y: zz.y + c.y };
        if (z.x * z.x + z.y * z.y > 4.0) break;
        n += 1;
    }
    return n / maxIter;
}

function renderMandelbrot(center, width, height, maxIter, xRange, yRange, zoomFactor, frameNum) {
    let png = new PNG({ width, height });
    // passing interfaces like tuples seems to use the variable name along with its values, if the
    // name matches an expected property on the expected struct

    for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
            let iterRatio = mandelbrot({
                x: center.x + xRange / Math.pow(zoomFactor, frameNum) * (x - width / 2 ) / width,
                y: center.y + yRange / Math.pow(zoomFactor, frameNum) * (y - height / 2) / height
            }, maxIter);

            let idx = (width * y + x) << 2;

            let color;
            if (iterRatio == 1) {
                color = { r: 0, g: 0, b: 0 };
            } else {
                color = sinebowColor(iterRatio);
            }

            png.data[idx] = color.r;
            png.data[idx + 1] = color.g;
            png.data[idx + 2] = color.b;
            png.data[idx + 3] = 255;
        }
    }
    let formattedFrameNum = String(frameNum).padStart(4, '0');
    return new Promise((resolve, reject) => {
        const stream = png.pack().pipe(fs.createWriteStream(`node_data/frame_${formattedFrameNum}.png`));
        // file saving is async, so the timing func might be 
        // inaccurate unless we wait for promise to resolve
        // This js way of (not having to) handling errors is disgusting and requires choice, and 
        // human memory/foresight
        stream.on('finish', resolve); 
        stream.on('error', reject);  
    });
}

var center = {
    x: -1.74999841099374081749002483162428393452822172335808534616943930976364725846655540417646727085571962736578151132907961927190726789896685696750162524460775546580822744596887978637416593715319388030232414667046419863755743802804780843375,
    y: -0.00000000000000165712469295418692325810961981279189026504290127375760405334498110850956047368308707050735960323397389547038231194872482690340369921750514146922400928554011996123112902000856666847088788158433995358406779259404221904755
};
var xRange = 4;
var yRange = 4;
var width = 1200;
var height = 1200;
var maxIter = 300;
var zoomFactor = 1.02;

var zoomStart = 0;
var zoomEnd = 100;

if (process.argv.length >= 6) {
    maxIter = Number(process.argv[2])
    zoomStart = Number(process.argv[3]);
    zoomEnd = Number(process.argv[4]);
    zoomFactor = Number(process.argv[5]);
}

if (cluster.isMaster) {
    let totalTimeMs = 0;
    let frameQueue = Array.from({ length: zoomEnd - zoomStart }, (_, i) => zoomStart + i);
    let workers = [];
  
    console.log(`Master ${process.pid} is running`);
  
    for (let i = 0; i < numCPUs; i++) {
      const worker = cluster.fork();
      workers.push(worker);
      if (frameQueue.length > 0) {
        let frameNum = frameQueue.shift();
        worker.send({ frameNum, maxIter, zoomFactor });
      }
    }
  
    cluster.on('exit', (worker, code, signal) => {
      console.log(`worker ${worker.process.pid} finished`);
      let workerIndex = workers.indexOf(worker);
      if (workerIndex >= 0) {
        workers.splice(workerIndex, 1);
      }
      if (workers.length == 0) {
        let totalTimeS = totalTimeMs / 1000.0
        let avgMsPerFrame = totalTimeMs / frameQueue.length;
        console.log(`Generated ${frameQueue.length} frames in ${totalTimeS} s at avg ${avgMsPerFrame} ms per frame.`);
        try {
            exec('ffmpeg -y -framerate 30 -i node_data/frame_%04d.png -c:v libx264 -pix_fmt yuv420p node_data/node_out.mp4');
        } catch (error) {
            console.log(`Error: ${error.message}`);
        }
    }
    });
  
    cluster.on('message', (worker, msg) => {
      if (msg.cmd && msg.cmd === 'notifyRenderTime') {
        totalTimeMs += msg.time;
        if (frameQueue.length > 0) {
          let frameNum = frameQueue.shift();
          worker.send({ frameNum, maxIter, zoomFactor });
        }
      }
    });
  
  } else {
    process.on('message', async ({ frameNum, maxIter, zoomFactor }) => {
      console.time(`frame ${frameNum}`);
      await renderMandelbrot(center, width, height, maxIter, xRange, yRange, zoomFactor, frameNum);
      let endTime = Date.now();
      console.timeEnd(`frame ${frameNum}`);
      process.send({ cmd: 'notifyRenderTime', time: endTime });
    });
  
    console.log(`Worker ${process.pid} started`);
  }
  