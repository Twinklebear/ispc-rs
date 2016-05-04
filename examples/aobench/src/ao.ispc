/* This example is based on Syoyo Fujita's aobench: https://github.com/syoyo/aobench
 * also see the Google code page: https://code.google.com/archive/p/aobench/
 */

#define NUM_AO_SAMPLES 8

typedef float<3> Vec3f;

struct Isect {
	float t;
	Vec3f p, n;
	int hit;
};

struct Sphere {
	Vec3f center;
	float radius;
};

struct Plane {
	Vec3f p, n;
};

struct Ray {
	Vec3f origin, dir;
};

inline float dot(Vec3f a, Vec3f b){
	return a.x * b.x + a.y * b.y + a.z * b.z;
}
inline Vec3f cross(Vec3f a, Vec3f b){
	Vec3f c;
    c.x = a.y * b.z - a.z * b.y;
    c.y = a.z * b.x - a.x * b.z;
    c.z = a.x * b.y - a.y * b.x;
	return c;
}
inline Vec3f normalized(Vec3f a){
	float inv_len = rsqrt(dot(a, a));
	return a * inv_len;
}

inline void sphere_intersect(Isect &isect, const Ray &ray, const uniform Sphere &sphere){
	Vec3f rs = ray.origin - sphere.center;

	const float b = dot(rs, ray.dir);
	const float c = dot(rs, rs) - sphere.radius * sphere.radius;
	const float discrim = b * b - c;
	if (discrim > 0.f){
		// Note: we will never hit the backface of the sphere
		const float t = -b - sqrt(discrim);
		if (t > 0.f && t < isect.t){
			isect.t = t;
			isect.hit = 1;

			isect.p = ray.origin + ray.dir * t;
			isect.n = normalized(isect.p - sphere.center);
		}
	}
}

inline void plane_intersect(Isect &isect, const Ray &ray, const uniform Plane &plane){
	const float d = -dot(plane.p, plane.n);
	const float v = dot(ray.dir, plane.n);
	if (abs(v) < 1e-6f){
		return;
	}

	const float t = -(dot(ray.origin, plane.n) + d) / v;
	if (t > 0.f && t < isect.t){
		isect.t = t;
		isect.hit = 1;

		isect.p = ray.origin + ray.dir * t;
		isect.n = plane.n;
	}
}

inline void ortho_basis(Vec3f basis[3], const Vec3f n){
	basis[2] = n;
	basis[1] = 0.f;

	if (n.x < 0.6f && n.x > -0.6f){
		basis[1].x = 1.f;
	} else if (n.y < 0.6f && n.y > -0.6f){
		basis[1].y = 1.f;
	} else if (n.z < 0.6f && n.z > -0.6f){
		basis[1].z = 1.f;
	} else {
		basis[1].x = 1.f;
	}
	basis[0] = normalized(cross(basis[1], basis[2]));
	basis[1] = normalized(cross(basis[2], basis[0]));
}

float ambient_occlusion(const Isect &isect, const uniform Sphere *uniform spheres,
		const uniform Plane &plane, RNGState &rng_state){
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

			sphere_intersect(occluded, ray, spheres[0]);
			sphere_intersect(occluded, ray, spheres[1]);
			sphere_intersect(occluded, ray, spheres[2]);
			plane_intersect(occluded, ray, plane);

			if (occluded.hit){
				occlusion += 1.f;
			}
		}
	}
	occlusion = (NUM_AO_SAMPLES * NUM_AO_SAMPLES - occlusion) / (NUM_AO_SAMPLES * NUM_AO_SAMPLES);
	return occlusion;
}
task void scanline(const uniform int w, const uniform int h, const uniform int n_samples, const uniform int seed,
		uniform float img[], const uniform Plane &plane, const uniform Sphere spheres[]){
	RNGState rng_state;
	// Make the seed sort of scanline dependent
	seed_rng(&rng_state, seed * (programIndex + taskIndex0 + 1));
	const float inv_samples = 1.f / n_samples;
	foreach (i = 0 ... w){
		for (int v = 0; v < n_samples; ++v){
			for (int u = 0; u < n_samples; ++u){
				const float px = (i + (u * inv_samples) - w / 2.f) / (w / 2.f);
				const float py = -(taskIndex0 + (v * inv_samples) - h / 2.f) / (h / 2.f);

				Ray ray;
				ray.origin = 0.f;
				ray.dir.x = px;
				ray.dir.y = py;
				ray.dir.z = -1.f;
				ray.dir = normalized(ray.dir);

				Isect isect;
				isect.t = 1e17f;
				isect.hit = 0;

				sphere_intersect(isect, ray, spheres[0]);
				sphere_intersect(isect, ray, spheres[1]);
				sphere_intersect(isect, ray, spheres[2]);
				plane_intersect(isect, ray, plane);
				if (isect.hit){
					const float col = ambient_occlusion(isect, spheres, plane, rng_state);
					// It's just a grayscale image
					img[taskIndex0 * w + i] += col;
				}
			}
		}
		img[taskIndex0 * w + i] *= inv_samples * inv_samples;
	}
}
export void aobench_parallel(const uniform int w, const uniform int h, const uniform int n_samples,
		const uniform int seed, uniform float img[]){
	const uniform Plane plane = {{0.f, -0.5f, 0.f}, {0.f, 1.f, 0.f}};
	const uniform Sphere spheres[3] = {
		{{-2.f, 0.f, -3.5f}, 0.5f},
		{{-0.5f, 0.f, -3.f}, 0.5f},
		{{1.f, 0.f, -2.2f}, 0.5f}
	};
	// For the parallel implementation we just render each scanline in parallel
	launch[h] scanline(w, h, n_samples, seed, img, plane, spheres);
}
export void aobench(const uniform int w, const uniform int h, const uniform int n_samples, const uniform int seed,
		uniform float img[]){
	const uniform Plane plane = {{0.f, -0.5f, 0.f}, {0.f, 1.f, 0.f}};
	const uniform Sphere spheres[3] = {
		{{-2.f, 0.f, -3.5f}, 0.5f},
		{{-0.5f, 0.f, -3.f}, 0.5f},
		{{1.f, 0.f, -2.2f}, 0.5f}
	};
	RNGState rng_state;
	seed_rng(&rng_state, seed * (programIndex + 1));
	const float inv_samples = 1.f / n_samples;

	// TODO: Tiled looping?
	for (uniform int j = 0; j < h; ++j){
		foreach (i = 0 ... w){
			for (int v = 0; v < n_samples; ++v){
				for (int u = 0; u < n_samples; ++u){
					const float px = (i + (u * inv_samples) - w / 2.f) / (w / 2.f);
					const float py = -(j + (v * inv_samples) - h / 2.f) / (h / 2.f);

					Ray ray;
					ray.origin = 0.f;
					ray.dir.x = px;
					ray.dir.y = py;
					ray.dir.z = -1.f;
					ray.dir = normalized(ray.dir);

					Isect isect;
					isect.t = 1e17f;
					isect.hit = 0;

					sphere_intersect(isect, ray, spheres[0]);
					sphere_intersect(isect, ray, spheres[1]);
					sphere_intersect(isect, ray, spheres[2]);
					plane_intersect(isect, ray, plane);
					if (isect.hit){
						const float col = ambient_occlusion(isect, spheres, plane, rng_state);
						// It's just a grayscale image
						img[j * w + i] += col;
					}
				}
			}
			img[j * w + i] *= inv_samples * inv_samples;
		}
	}
}

