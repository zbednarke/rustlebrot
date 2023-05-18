package main

import (
	"fmt"
	"image"
	"image/color"
	"image/png"
	"log"
	"math"
	"math/cmplx"
	"os"
	"os/exec"
	"runtime"
	"strconv"
	"sync"
	"time"
)

type FrameJob struct {
	frame      int
	xRange     [2]float64
	yRange     [2]float64
	zoomFactor float64
}

func mandelbrot(c complex128, maxIter int) float64 {
	z := c
	var n int
	for ; n < maxIter; n++ {
		if cmplx.Abs(z) > 2 {
			break
		}
		z = z*z + c
	}
	return float64(n) / float64(maxIter)
}

func colorGradient(iterRatio float64) color.RGBA {
	t := uint8(iterRatio * 255)
	return color.RGBA{t, t, 255 - t, 255}
}

func renderMandelbrot(width, height, maxIter int, xRange, yRange [2]float64) *image.RGBA {
	img := image.NewRGBA(image.Rect(0, 0, width, height))
	scalex := (xRange[1] - xRange[0]) / float64(width)
	scaley := (yRange[1] - yRange[0]) / float64(height)

	for y := 0; y < height; y++ {
		for x := 0; x < width; x++ {
			c := complex(float64(x)*scalex+xRange[0], float64(y)*scaley+yRange[0])
			iterRatio := mandelbrot(c, maxIter)
			img.Set(x, y, colorGradient(iterRatio))
		}
	}
	return img
}

func workerFrame(jobs <-chan FrameJob, maxIter, width, height int) {
	for j := range jobs {
		startTime := time.Now()

		img := renderMandelbrot(width, height, maxIter, j.xRange, j.yRange)

		outputFilename := fmt.Sprintf("mandelbrot_set_%04d.png", j.frame)
		outputFile, _ := os.Create(outputFilename)
		defer outputFile.Close()

		png.Encode(outputFile, img)

		elapsedTime := time.Since(startTime)
		fmt.Printf("Frame %d completed in %v\n", j.frame, elapsedTime)

		xCenter := (j.xRange[0] + j.xRange[1]) / 2.0
		yCenter := (j.yRange[0] + j.yRange[1]) / 2.0
		xRangeWidth := (j.xRange[1] - j.xRange[0]) / j.zoomFactor
		yRangeWidth := (j.yRange[1] - j.yRange[0]) / j.zoomFactor

		j.xRange = [2]float64{xCenter - xRangeWidth/2.0, xCenter + xRangeWidth/2.0}
		j.yRange = [2]float64{yCenter - yRangeWidth/2.0, yCenter + yRangeWidth/2.0}
	}
}

func main() {
	maxIter, _ := strconv.Atoi(os.Args[1])
	zoomStart, _ := strconv.Atoi(os.Args[2])
	zoomEnd, _ := strconv.Atoi(os.Args[3])
	zoomFactor, _ := strconv.ParseFloat(os.Args[4], 64)

	width, height := 1200, 1200
	xCenter := -1.74999841099374081749002483162428393452822172335808534616943930976364725846655540417646727085571962736578151132907961927190726789896685696750162524460775546580822744596887978637416593715319388030232414667046419863755743802804780843375
	yCenter := -0.00000000000000165712469295418692325810961981279189026504290127375760405334498110850956047368308707050735960323397389547038231194872482690340369921750514146922400928554011996123112902000856666847088788158433995358406779259404221904755
	initialZoomFactor := math.Pow(zoomFactor, float64(zoomStart))
	var wg sync.WaitGroup

	xRange := [2]float64{
		xCenter - 2.0/initialZoomFactor,
		xCenter + 2.0/initialZoomFactor,
	}

	yRange := [2]float64{
		yCenter - 2.0/initialZoomFactor,
		yCenter + 2.0/initialZoomFactor,
	}

	jobs := make(chan FrameJob, zoomEnd-zoomStart+1)

	fmt.Printf("About to construct jobs\n")
	startTime := time.Now()

	for w := 0; w < runtime.NumCPU(); w++ {
		wg.Add(1)
		go func() {
			workerFrame(jobs, maxIter, width, height)
			wg.Done()
		}()
	}

	for frame := zoomStart; frame <= zoomEnd; frame++ {
		jobs <- FrameJob{frame, xRange, yRange, zoomFactor}
	}

	close(jobs)

	wg.Wait()

	elapsedTime := time.Since(startTime)
	fmt.Printf("%d frames completed in %v\n", zoomEnd-zoomStart+1, elapsedTime)
	fmt.Printf("Average time per frame: %f ms.\n", float64(elapsedTime.Milliseconds())/float64(zoomEnd-zoomStart+1))

	fmt.Printf("Jobs finished. About to render mp4\n")

	cmd := exec.Command("ffmpeg", "-y", "-framerate", "30", "-i", "go_data/mandelbrot_set_%04d.png", "-c:v", "libx264", "-pix_fmt", "yuv420p", "go_out.mp4")
	err := cmd.Run()
	if err != nil {
		log.Fatalf("Failed to execute command: %s", err)
	}
}
