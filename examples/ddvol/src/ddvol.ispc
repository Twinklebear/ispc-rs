#include "vec3f.ih"
#include "camera.ih"
#include "vol.ih"

task void render_scanline(const uniform Camera* uniform camera, const uniform ISPCHandle volume,
		const uniform int32 * uniform seeds, const uniform int32 width, const uniform int32 height,
		uniform float img[])
{
	RNGState rng_state;
	// On AVX1.1 and up we can use hardware randomness to improve seed quality
#if defined(ISPC_TARGET_AVX11) || defined(ISPC_TARGET_AVX2) || defined(ISPC_TARGET_AVX2) \
	|| defined(ISPC_TARGET_AVX512KNL)
	{
		int rand_val = 0;
		while (!rdrand(&rand_val));
		seed_rng(&rng_state, seeds[taskIndex0] + rand_val);
	}
#else
	seed_rng(&rng_state, seeds[taskIndex0] + programIndex);
#endif
	const uniform int n_samples = 16;
	const uniform float inv_samples = 1.f / n_samples;
	// TODO: Should switch and go parallel on each pixel and do n * programCount samples so
	// we're tracing packets. This will require a reduce at the end though to write the
	// samples back to the framebuffer without trampling.
	foreach (i = 0 ... width){
		for (int s = 0; s < n_samples; ++s){
			const float samples[2] = {frandom(&rng_state), frandom(&rng_state)};
			Ray ray = camera_ray(camera, i, taskIndex0, samples);
			float<4> color = raycast_volume(volume, ray);
			img[(taskIndex0 * width + i) * 3] += color.x;
			img[(taskIndex0 * width + i) * 3 + 1] += color.y;
			img[(taskIndex0 * width + i) * 3 + 2] += color.z;
		}
		for (int c = 0; c < 3; ++c){
			img[(taskIndex0 * width + i) * 3 + c] *= inv_samples;
		}
	}
}
// Raycast the volume
export void render(const uniform ISPCHandle camera, const uniform ISPCHandle volume,
		const uniform int32 * uniform seeds, const uniform int32 width, const uniform int32 height,
		uniform float img[])
{
	const uniform Camera* uniform cam = (const uniform Camera* uniform)camera;
	launch[height] render_scanline(cam, volume, seeds, width, height, img);
}
// We don't use ISPC's build in srgb conversion as it seems to be kind of buggy.
// Maybe it assumes some alignment of the data that we don't enforce?
float linear_to_srgb(const float f) {
	if (f <= 0.0031308){
		return 12.92 * f;
	} else {
		return 1.055 * pow(f, 1.0 / 2.4) - 0.055;
	}
}
task void scanline_to_srgb(const uniform float fb[], uniform unsigned int8 srgb[], const uniform int32 width){
	foreach (i = 0 ... width){
		for (int c = 0; c < 3; ++c){
			float val = linear_to_srgb(fb[(taskIndex0 * width + i) * 3 + c]);
			srgb[(taskIndex0 * width + i) * 3 + c] = clamp(val * 255.0, 0.0, 255.0);
		}
	}
}
// Convert the linear RGB framebuffer we render to to sRGB8 for saving out to an image
export void framebuffer_to_srgb(const uniform float fb[], uniform unsigned int8 srgb[],
		const uniform int32 width, const uniform int32 height)
{
	launch[height] scanline_to_srgb(fb, srgb, width);
}
