/* This is a simple packetized ray tracer example which demonstrates
 * interopability with structs in Rust and ISPC.
 */

#include "vec3f.ih"
#include "lights.ih"
#include "material.ih"
#include "geom.ih"

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
task void render_scanline(const uniform Camera * uniform camera,
		const uniform Geometry * const uniform * uniform geom,
		const uniform int32 n_geom, const uniform Light * uniform light, const uniform int32 seed,
		const uniform int32 width, const uniform int32 height, uniform float img[])
{
	RNGState rng_state;
	// Make the seed sort of scanline dependent
	seed_rng(&rng_state, seed * (programIndex + taskIndex0 + 1));
	const uniform int n_samples = 256;
	const uniform float inv_samples = 1.f / n_samples;
	// TODO: Should switch and go parallel on each pixel and do n * programCount samples
	foreach (i = 0 ... width){
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
				img[(taskIndex0 * width + i) * 3] += color.x;
				img[(taskIndex0 * width + i) * 3 + 1] += color.y;
				img[(taskIndex0 * width + i) * 3 + 2] += color.z;
			}
		}
		for (int c = 0; c < 3; ++c){
			img[(taskIndex0 * width + i) * 3 + c] *= inv_samples;
		}
	}
}
export void render(const uniform Camera * uniform camera, const uniform Geometry * const uniform * uniform geom,
		const uniform int32 n_geom, const uniform Light * uniform light, const uniform int32 seed,
		const uniform int32 width, const uniform int32 height, uniform float img[])
{
	launch[height] render_scanline(camera, geom, n_geom, light, seed, width, height, img);
}

