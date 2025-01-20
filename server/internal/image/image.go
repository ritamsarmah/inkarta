package image

import (
	"bytes"
	"image"
	"image/color"
	_ "image/gif"
	_ "image/jpeg"
	_ "image/png"
	"inkarta/internal/db"
	"log/slog"
	"math"
	"mime/multipart"

	"golang.org/x/image/bmp"
	"golang.org/x/image/draw"
)

// Converts an image into a grayscale bitmap.
func Process(f *multipart.File) ([]byte, error) {
	src, _, err := image.Decode(*f)
	if err != nil {
		return nil, err
	}

	bounds := src.Bounds()
	dst := image.NewGray(bounds)
	draw.FloydSteinberg.Draw(dst, bounds, src, image.Point{})

	var buffer bytes.Buffer
	if err := bmp.Encode(&buffer, dst); err != nil {
		return nil, err
	}

	return buffer.Bytes(), nil
}

// Resizes image to desired dimensions.
func Resize(img *db.Image, newWidth int, newHeight int) *bytes.Buffer {
	reader := bytes.NewReader(img.Data)
	src, _, _ := image.Decode(reader)

	oldWidth := src.Bounds().Dx()
	oldHeight := src.Bounds().Dy()

	if newWidth == 0 {
		newWidth = oldWidth
	}

	if newHeight == 0 {
		newHeight = oldHeight
	}

	var buffer bytes.Buffer
	if newWidth != oldWidth || newHeight != oldHeight {
		slog.Info("Returning resized image", "title", img.Title, "width", newWidth, "height", newHeight)

		// Determine fill color
		var fill color.Color
		if img.Dark {
			fill = color.Black
		} else {
			fill = color.White
		}

		// Create destination canvas with background fill
		dst := image.NewGray(image.Rect(0, 0, newWidth, newHeight))
		draw.Draw(dst, dst.Bounds(), &image.Uniform{fill}, image.Point{}, draw.Src)

		// Calculate the scaling factor to maintain aspect ratio
		scaleX := float64(newWidth) / float64(oldWidth)
		scaleY := float64(newHeight) / float64(oldHeight)
		scale := math.Min(scaleX, scaleY)

		// Calculate scaled image dimensions
		scaledWidth := int(float64(oldWidth) * scale)
		scaledHeight := int(float64(oldHeight) * scale)

		// Calculate offsets to center the scaled image
		offsetX := (newWidth - scaledWidth) / 2
		offsetY := (newHeight - scaledHeight) / 2

		// Scale the source image into the destination
		scaledRect := image.Rect(offsetX, offsetY, offsetX+scaledWidth, offsetY+scaledHeight)
		draw.ApproxBiLinear.Scale(dst, scaledRect, src, src.Bounds(), draw.Over, nil)
		bmp.Encode(&buffer, dst)
	} else {
		slog.Info("Returning full resolution image", "title", img.Title, "width", oldWidth, "height", oldHeight)
		bmp.Encode(&buffer, src)
	}

	return &buffer
}
