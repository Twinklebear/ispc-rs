#include "vec3f.ih"
#include "camera.ih"
#include "vol.ih"
#include "fb.ih"

struct RenderParams {
	Vec3f background;
	int n_samples;
};

export void render_tile(const uniform ISPCHandle cam, const uniform ISPCHandle volume,
		const uniform RenderParams* uniform params, const uniform int32 seed, uniform Tile* uniform tile)
{
	const uniform Camera* uniform camera = (const uniform Camera* uniform)cam;
	RNGState rng_state;
	// On AVX1.1 and up we can use hardware randomness to improve seed quality
#if defined(ISPC_TARGET_AVX11) || defined(ISPC_TARGET_AVX2) || defined(ISPC_TARGET_AVX2) \
	|| defined(ISPC_TARGET_AVX512KNL)
	{
		int rand_val = 0;
		while (!rdrand(&rand_val));
		seed_rng(&rng_state, seed + rand_val);
	}
#else
	seed_rng(&rng_state, seed + programIndex);
#endif
	const uniform float inv_samples = 1.f / params->n_samples;
	// TODO: Should switch and go parallel on each pixel and do n * programCount samples so
	// we're tracing packets. This will require a reduce at the end though to write the
	// samples back to the framebuffer without trampling.
	const float<4> background = {params->background.x, params->background.y, params->background.z, 1};
	foreach_tiled (y = 0 ... tile->tile_size, x = 0 ... tile->tile_size){
		const int<2> pixel = {tile->x + x, tile->y + y};
		const int idx = (pixel.y * tile->tile_size + pixel.x) * 4;
		for (uniform int s = 0; s < params->n_samples; ++s){
			const float samples[2] = {frandom(&rng_state), frandom(&rng_state)};
			Ray ray = camera_ray(camera, pixel.x, pixel.y, samples);
			const float<4> color = raycast_volume(volume, ray);
			// Composite the volume color over the background color and accumulate into
			// the framebuffer
			const float<3> composite = color.xyz + (1.0 - color.w) * background.w * background.xyz;
			tile->data[idx] += composite.x;
			tile->data[idx + 1] += composite.y;
			tile->data[idx + 2] += composite.z;
			tile->data[idx + 3] += color.w;
		}
		for (int c = 0; c < 4; ++c){
			tile->data[idx + c] *= inv_samples;
		}
	}
}
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
		const int idx = (taskIndex0 * width + i) * 4;
		for (uniform int s = 0; s < params->n_samples; ++s){
			const float samples[2] = {frandom(&rng_state), frandom(&rng_state)};
			Ray ray = camera_ray(camera, i, taskIndex0, samples);
			const float<4> color = raycast_volume(volume, ray);
			// Composite the volume color over the background color and accumulate into
			// the framebuffer
			const float<3> composite = color.xyz + (1.0 - color.w) * background.w * background.xyz;
			img[idx] += composite.x;
			img[idx + 1] += composite.y;
			img[idx + 2] += composite.z;
			img[idx + 3] += color.w;
		}
		for (int c = 0; c < 4; ++c){
			img[idx + c] *= inv_samples;
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

