/* This is a simple packetized ray tracer example which demonstrates
 * interopability with structs in Rust and ISPC.
 */

#include "vec3f.ih"
#include "lights.ih"
#include "material.ih"
#include "geom.ih"

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
			if (sphere_intersect(isect, ray, geom[i])){
				isect.hit = i + 1;
			}
		} else if (geom[i]->type == PLANE){
			if (plane_intersect(isect, ray, geom[i])){
				isect.hit = i + 1;
			}
		}
	}
}
float ambient_occlusion(const uniform Geometry * const uniform * uniform geom, const uniform int32 n_geom,
		const Isect &isect, RNGState &rng_state)
{
	const Vec3f p = isect.p + 0.0001f * isect.n;
	Vec3f basis[3];
	ortho_basis(basis, isect.n);
	float occlusion = 0.f;

	for (int j = 0; j < NUM_AO_SAMPLES; ++j){
		for (int i = 0; i < NUM_AO_SAMPLES; ++i){
			const float theta = sqrt(frandom(&rng_state));
			const float phi = 2.0f * PI * frandom(&rng_state);

			const float x = cos(phi) * theta;
			const float y = sin(phi) * theta;
			const float z = sqrt(1.f - theta * theta);

			// Transform from object space to world space
			Ray ray;
			ray.origin = p;
			ray.dir.x = x * basis[0].x + y * basis[1].x + z * basis[2].x;
			ray.dir.y = x * basis[0].y + y * basis[1].y + z * basis[2].y;
			ray.dir.z = x * basis[0].z + y * basis[1].z + z * basis[2].z;

			Isect occluded;
			occluded.t = 1e17f;
			occluded.hit = 0;
			intersect_scene(occluded, ray, geom, n_geom);
			if (occluded.hit){
				occlusion += 1.f;
			}
		}
	}
	occlusion = (NUM_AO_SAMPLES * NUM_AO_SAMPLES - occlusion) / (NUM_AO_SAMPLES * NUM_AO_SAMPLES);
	return occlusion;
}
task void render_scanline(const uniform Camera * uniform camera,
		const uniform Geometry * const uniform * uniform geom,
		const uniform int32 n_geom, const uniform Light * uniform light, const uniform int32 seed,
		const uniform int32 width, const uniform int32 height, uniform unsigned int8 img[])
{
	RNGState rng_state;
	// Make the seed sort of scanline dependent
	seed_rng(&rng_state, seed * (programIndex + taskIndex0 + 1));
	const uniform int n_samples = 64;
	const uniform float inv_samples = 1.f / n_samples;
	// TODO: Should switch and go parallel on each pixel and do n * programCount samples
	foreach (i = 0 ... width){
		float pixel_color[3] = {0};
		for (int s = 0; s < n_samples; ++s){
			Isect isect;
			isect.t = 1e30f;
			isect.hit = 0;
			const float samples[2] = {frandom(&rng_state), frandom(&rng_state)};

			Ray ray = camera_ray(camera, i, taskIndex0, samples);
			intersect_scene(isect, ray, geom, n_geom);
			Vec3f ambient_light = make_vec3f(0.1, 0.1, 0.1);
			if (isect.hit){
				Vec3f emission, light_dir;
				point_light_incident(light, isect.p, emission, light_dir);

				Ray shadow;
				shadow.origin = isect.p + 0.0001 * isect.n;
				shadow.dir = light_dir;
				Isect shadow_hit;
				shadow_hit.t = 1e30f;
				shadow_hit.hit = 0;
				intersect_scene(shadow_hit, shadow, geom, n_geom);

				float geom_term = dot(isect.n, light_dir);
				// Some base ambient lighting
				Vec3f color = ambient_light * geom[isect.hit - 1]->material->albedo;
				if (!shadow_hit.hit && geom_term > 0.0){
					Vec3f w_o = negate(ray.dir);
					// Just the Lambertian material for now
					Vec3f bsdf_f = shade_lambertian(geom[isect.hit - 1]->material, w_o, light_dir);
					color = color + emission * geom_term * bsdf_f;
				}
				color = color * ambient_occlusion(geom, n_geom, isect, rng_state);
				pixel_color[0] += color.x;
				pixel_color[1] += color.y;
				pixel_color[2] += color.z;
			}
		}
		for (int c = 0; c < 3; ++c){
			pixel_color[c] *= inv_samples;
			if (pixel_color[c] > 1.f){
				pixel_color[c] = 1.f;
			} else if (pixel_color[c] < 0.f){
				pixel_color[c] = 0.f;
			}
			// Convert the pixel to sRGB and write the final values to the framebuffer, since no reconstruction
			// filtering is done we don't need to maintain a linear RGB framebuffer and convert to sRGB at the end.
			img[(taskIndex0 * width + i) * 3 + c] = (unsigned int8)float_to_srgb8(pixel_color[c]);
		}
	}
}
// Render the scene with Whitted raytracing + AO to an sRGB image
export void render(const uniform Camera * uniform camera, const uniform Geometry * const uniform * uniform geom,
		const uniform int32 n_geom, const uniform Light * uniform light, const uniform int32 seed,
		const uniform int32 width, const uniform int32 height, uniform unsigned int8 img[])
{
	launch[height] render_scanline(camera, geom, n_geom, light, seed, width, height, img);
}

