/* This is a simple packetized ray tracer example which demonstrates
 * interopability with structs in Rust and ISPC.
 */

#include "vec3f.ih"
#include "lights.ih"
#include "material.ih"
#include "geom.ih"
#include "mc.ih"

#define NUM_AO_SAMPLES 8

struct Camera {
	// Specify the location of the camera in the world
	Vec3f pos, dir, up;
	// Computed values to make finding the ray through a pixel easier
	Vec3f dir_top_left, screen_du, screen_dv;
	int32 width, height;
};
/* Generate a jittered ray through pixel x,y using the samples to offset randomly within
 * the pixel. samples should be in [0, 1]
 */
Ray camera_ray(const uniform Camera * uniform cam, const float x, const float y, const float samples[2]){
	Ray ray;
	ray.origin = cam->pos;
	ray.dir = cam->dir_top_left;
	const Vec3f u_step = ((x + samples[0]) / cam->width) * cam->screen_du;
	const Vec3f v_step = ((y + samples[1]) / cam->height) * cam->screen_dv;
	ray.dir = ray.dir + u_step + v_step;
	ray.dir = normalize(ray.dir);
	return ray;
}

void intersect_scene(Isect &isect, const Ray &ray, const uniform Geometry * const uniform * uniform geom,
		const uniform int32 n_geom)
{
	for (uniform int i = 0; i < n_geom; ++i){
		if (geom[i]->type == SPHERE){
			if (sphere_intersect(geom[i], isect, ray)){
				isect.hit = i + 1;
			}
		} else if (geom[i]->type == PLANE){
			if (plane_intersect(geom[i], isect, ray)){
				isect.hit = i + 1;
			}
		}
	}
}
Vec3f pathtracer_li(const Ray &r, const uniform Geometry * const uniform * uniform geom,
		const uniform int32 n_geom, const uniform Light * uniform light, RNGState rng_state)
{
	Vec3f color = make_vec3f(0, 0, 0);
	Vec3f path_throughput = make_vec3f(1, 1, 1);
	Ray ray = r;
	Vec3f basis[3];
	const int max_depth = 10;
	for (int i = 0; i < max_depth; ++i){
		Isect isect;
		isect.t = 1e30f;
		isect.hit = 0;
		intersect_scene(isect, ray, geom, n_geom);
		if (isect.hit){
			Vec3f emission, light_dir;
			point_light_incident(light, isect.p, emission, light_dir);

			Ray shadow;
			Isect shadow_hit;
			shadow.origin = isect.p + 0.001 * isect.n;
			point_light_occlusion_test(light, shadow, shadow_hit);
			intersect_scene(shadow_hit, shadow, geom, n_geom);

			Vec3f w_o = negate(ray.dir);
			// Just the Lambertian material for now
			Vec3f bsdf_f = shade_lambertian(geom[isect.hit - 1]->material, w_o, light_dir);
			if (!shadow_hit.hit){
				color = color + path_throughput * emission * abs(dot(isect.n, light_dir)) * bsdf_f;
			}
			Vec3f hemi_sample = cos_sample_hemisphere(frandom(&rng_state), frandom(&rng_state));
			ortho_basis(basis, isect.n);
			Vec3f w_i;
			w_i.x = hemi_sample.x * basis[0].x + hemi_sample.y * basis[1].x + hemi_sample.z * basis[2].x;
			w_i.y = hemi_sample.x * basis[0].y + hemi_sample.y * basis[1].y + hemi_sample.z * basis[2].y;
			w_i.z = hemi_sample.x * basis[0].z + hemi_sample.y * basis[1].z + hemi_sample.z * basis[2].z;
			w_i = normalize(w_i);
			float bsdf_pdf = pdf_lambertian(geom[isect.hit - 1]->material, w_o, w_i, isect.n);
			if (length(bsdf_f) == 0.0 || bsdf_pdf == 0.0){
				break;
			}
			path_throughput = path_throughput * bsdf_f * abs(dot(w_i, isect.n)) / bsdf_pdf;
			// Transform from object space to world space
			ray.origin = shadow.origin;
			ray.dir = w_i;
		} else {
			break;
		}
	}
	return color;
}

task void render_scanline(const uniform Camera * uniform camera,
		const uniform Geometry * const uniform * uniform geom,
		const uniform int32 n_geom, const uniform Light * uniform light, const uniform int32 * uniform seeds,
		const uniform int32 width, const uniform int32 height, uniform float img[])
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
	const uniform int n_samples = 64;
	const uniform float inv_samples = 1.f / n_samples;
	// TODO: Should switch and go parallel on each pixel and do n * programCount samples
	foreach (i = 0 ... width){
		for (int s = 0; s < n_samples; ++s){
			const float samples[2] = {frandom(&rng_state), frandom(&rng_state)};
			Ray ray = camera_ray(camera, i, taskIndex0, samples);
			Vec3f color = pathtracer_li(ray, geom, n_geom, light, rng_state);
			img[(taskIndex0 * width + i) * 3] += color.x;
			img[(taskIndex0 * width + i) * 3 + 1] += color.y;
			img[(taskIndex0 * width + i) * 3 + 2] += color.z;
		}
		for (int c = 0; c < 3; ++c){
			img[(taskIndex0 * width + i) * 3 + c] *= inv_samples;
		}
	}
}
// Render the scene with Whitted raytracing + AO to an sRGB image
export void render(const uniform Camera * uniform camera, const uniform Geometry * const uniform * uniform geom,
		const uniform int32 n_geom, const uniform Light * uniform light, const uniform int32 * uniform seeds,
		const uniform int32 width, const uniform int32 height, uniform float img[])
{
	launch[height] render_scanline(camera, geom, n_geom, light, seeds, width, height, img);
}
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

