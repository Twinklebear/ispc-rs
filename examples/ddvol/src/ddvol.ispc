#include "vec3f.ih"
#include "camera.ih"
#include "vol.ih"

struct RenderParams {
	Vec3f background;
	int n_samples;
};

task void render_scanline(const uniform Camera* uniform camera, const uniform ISPCHandle volume,
		const uniform RenderParams* uniform params, const uniform int32 * uniform seeds,
		const uniform uint32_t width, const uniform uint32_t height, uniform float img[])
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
	const uniform float inv_samples = 1.f / params->n_samples;
	// TODO: Should switch and go parallel on each pixel and do n * programCount samples so
	// we're tracing packets. This will require a reduce at the end though to write the
	// samples back to the framebuffer without trampling.
	const float<4> background = {params->background.x, params->background.y, params->background.z, 1};
	foreach (i = 0 ... width){
		for (int s = 0; s < params->n_samples; ++s){
			const float samples[2] = {frandom(&rng_state), frandom(&rng_state)};
			Ray ray = camera_ray(camera, i, taskIndex0, samples);
			const float<4> color = raycast_volume(volume, ray);
			// Composite the volume color over the background color and accumulate into
			// the framebuffer
			const float<3> composite = color.xyz + (1.0 - color.w) * background.w * background.xyz;
			img[(taskIndex0 * width + i) * 3] += composite.x;
			img[(taskIndex0 * width + i) * 3 + 1] += composite.y;
			img[(taskIndex0 * width + i) * 3 + 2] += composite.z;
		}
		for (int c = 0; c < 3; ++c){
			img[(taskIndex0 * width + i) * 3 + c] *= inv_samples;
		}
	}
}
// Raycast the volume
export void render(const uniform ISPCHandle camera, const uniform ISPCHandle volume,
		const uniform RenderParams* uniform params, const uniform int32 * uniform seeds,
		const uniform uint32_t width, const uniform uint32_t height, uniform float img[])
{
	const uniform Camera* uniform cam = (const uniform Camera* uniform)camera;
	launch[height] render_scanline(cam, volume, params, seeds, width, height, img);
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
task void scanline_to_srgb(const uniform float fb[], uniform unsigned int8 srgb[], const uniform uint32_t width){
	foreach (i = 0 ... width){
		for (int c = 0; c < 3; ++c){
			float val = linear_to_srgb(fb[(taskIndex0 * width + i) * 3 + c]);
			srgb[(taskIndex0 * width + i) * 3 + c] = clamp(val * 255.0, 0.0, 255.0);
		}
	}
}
// Convert the linear RGB framebuffer we render to to sRGB8 for saving out to an image
export void framebuffer_to_srgb(const uniform float fb[], uniform unsigned int8 srgb[],
		const uniform uint32_t width, const uniform uint32_t height)
{
	launch[height] scanline_to_srgb(fb, srgb, width);
	// Flip the image about y
	for (uniform int y = 0; y < height / 2; ++y){
		foreach (x = 0 ... width * 3){
			const unsigned int8 tmp = srgb[y * width * 3 + x];
			srgb[y * width * 3 + x] = srgb[(height - y - 1) * width * 3 + x];
			srgb[(height - y - 1) * width * 3 + x] = tmp;
		}
	}
}
