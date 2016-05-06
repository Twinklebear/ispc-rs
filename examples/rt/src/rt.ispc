/* This is a simple packetized ray tracer example which demonstrates
 * interopability with structs in Rust and ISPC.
 */

struct Vec3f {
	float x, y, z;
};
inline Vec3f make_vec3f(float x, float y, float z){
	Vec3f v;
	v.x = x;
	v.y = y;
	v.z = z;
	return v;
}
inline Vec3f operator*(float s, Vec3f v){
	Vec3f res;
	res.x = v.x * s;
	res.y = v.y * s;
	res.z = v.z * s;
	return res;
}
inline Vec3f operator*(Vec3f v, float s){
	Vec3f res;
	res.x = v.x * s;
	res.y = v.y * s;
	res.z = v.z * s;
	return res;
}
inline Vec3f operator+(Vec3f a, Vec3f b){
	Vec3f res;
	res.x = a.x + b.x;
	res.y = a.y + b.y;
	res.z = a.z + b.z;
	return res;
}
inline Vec3f operator-(Vec3f a, Vec3f b){
	Vec3f res;
	res.x = a.x - b.x;
	res.y = a.y - b.y;
	res.z = a.z - b.z;
	return res;
}
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
inline Vec3f normalize(Vec3f a){
	float inv_len = rsqrt(dot(a, a));
	return a * inv_len;
}

struct Ray {
	Vec3f origin, dir;
};

struct Isect {
	float t;
	Vec3f p, n;
	int hit;
};

struct Sphere {
	Vec3f center;
	float radius;
};

inline void sphere_intersect(Isect &isect, const Ray &ray, const uniform Sphere &sphere){
	Vec3f rs = ray.origin - sphere.center;
	const float b = dot(rs, ray.dir);
	const float c = dot(rs, rs) - sphere.radius * sphere.radius;
	const float discrim = b * b - c;
	if (discrim > 0.f){
		float t = 0.5f * (-b - sqrt(discrim));
		if (t <= 0.f || t >= isect.t){
			t = 0.5f * (-b + sqrt(discrim));
		}
		if (t > 0.f && t < isect.t){
			isect.t = t;
			isect.hit = 1;
			isect.p = ray.origin + ray.dir * t;
			isect.n = normalize(isect.p - sphere.center);
		}
	}
}

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

export void render(const uniform Camera * uniform camera, const uniform Sphere * uniform sphere,
		const uniform int32 seed, const uniform int32 width, const uniform int32 height, uniform float img[])
{
	RNGState rng_state;
	seed_rng(&rng_state, seed * (programIndex + 1));
	const uniform int n_samples = 256;
	const uniform float inv_samples = 1.f / n_samples;
	for (uniform int j = 0; j < height; ++j){
		// TODO: Should switch and go parallel on each pixel and do n * programCount samples
		foreach (i = 0 ... width){
			for (int s = 0; s < n_samples; ++s){
				Isect isect;
				isect.t = 1e30f;
				isect.hit = 0;
				const float samples[2] = {frandom(&rng_state), frandom(&rng_state)};

				Ray ray = camera_ray(camera, i, j, samples);
				sphere_intersect(isect, ray, *sphere);
				if (isect.hit){
					for (int c = 0; c < 3; ++c){
						img[(j * width + i) * 3 + c] += 1.f;
					}
				}
			}
			for (int c = 0; c < 3; ++c){
				img[(j * width + i) * 3 + c] *= inv_samples;
			}
		}
	}
}

