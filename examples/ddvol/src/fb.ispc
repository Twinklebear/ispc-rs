#include "fb.ih"

// We don't use ISPC's build in srgb conversion as it seems to be kind of buggy.
// Maybe it assumes some alignment of the data that we don't enforce?
float linear_to_srgb(const float f) {
	if (f <= 0.0031308){
		return 12.92 * f;
	} else {
		return 1.055 * pow(f, 1.0 / 2.4) - 0.055;
	}
}
task void scanline_to_srgb(const uniform float fb[], uniform unsigned int8 srgb[], const uniform uint32_t width){
	foreach_tiled (i = 0 ... width, c = 0 ... 3){
		const float val = linear_to_srgb(fb[(taskIndex0 * width + i) * 4 + c]);
		srgb[(taskIndex0 * width + i) * 3 + c] = clamp(val * 255.0, 0.0, 255.0);
	}
}
// Convert the linear RGB framebuffer we render to to sRGB8 for saving out to an image
export void framebuffer_to_srgb(const uniform float fb[], uniform unsigned int8 srgb[],
		const uniform uint32_t width, const uniform uint32_t height)
{
	launch[height] scanline_to_srgb(fb, srgb, width);
	sync;
	// Flip the image about y
	for (uniform int y = 0; y < height / 2; ++y){
		foreach (x = 0 ... width * 3){
			const unsigned int8 tmp = srgb[y * width * 3 + x];
			srgb[y * width * 3 + x] = srgb[(height - y - 1) * width * 3 + x];
			srgb[(height - y - 1) * width * 3 + x] = tmp;
		}
	}
}


