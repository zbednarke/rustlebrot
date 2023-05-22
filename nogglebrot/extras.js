
// ( async function() {
//     // wtf is this inline funciton syntax 
//     let startTime = Date.now();
//     for (let frameNum = zoomStart; frameNum < zoomEnd; frameNum++) {
//         console.time(`frame ${frameNum}`);
//         await renderMandelbrot(center, width, height, maxIter, xRange, yRange, zoomFactor, frameNum);
//         console.timeEnd(`frame ${frameNum}`);
//     }
//     let endTime = Date.now();
//     let nFrames = zoomEnd - zoomStart
//     let totalTimeMs = endTime - startTime
//     let totalTimeS = totalTimeMs / 1000.0
//     let avgMsPerFrame = totalTimeMs / nFrames

//     exec('ffmpeg -y -framerate 30 -i node_data/frame_%04d.png -c:v libx264 -pix_fmt yuv420p node_data/node_out.mp4', (error, stdout, stderr) => {
//         if (error) {
//             console.log(`Error: ${error.message}`);
//             return;
//         }
//         if (stderr) {
//             console.log(`Stderr: ${stderr}`);
//             return;
//         }
//         console.log(`Stdout: ${stdout}`);
//     });

//     console.log(`Generated ${nFrames} frames in ${totalTimeS} s at avg ${avgMsPerFrame} ms per frame.`)
// })();

