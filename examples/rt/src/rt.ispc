/* This is a simple packetized ray tracer example which demonstrates
 * interopability with structs in Rust and ISPC.
 */

#define M_1_PI 0.31830988618379067154

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
inline Vec3f operator*(Vec3f a, Vec3f b){
	Vec3f res;
	res.x = a.x * b.x;
	res.y = a.y * b.y;
	res.z = a.z * b.z;
	return res;
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
inline Vec3f operator/(Vec3f v, float s){
	Vec3f res;
	res.x = v.x / s;
	res.y = v.y / s;
	res.z = v.z / s;
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
inline float length(Vec3f a){
	return sqrt(dot(a, a));
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
inline Vec3f negate(Vec3f a){
	Vec3f res;
	res.x = -a.x;
	res.y = -a.y;
	res.z = -a.z;
	return res;
}

struct Ray {
	Vec3f origin, dir;
};

struct Isect {
	float t;
	Vec3f p, n;
	int hit;
};

enum LIGHT_TYPE {
	POINT
};

struct Light {
	LIGHT_TYPE type;
};

struct PointLight {
	Light light;
	Vec3f position;
	Vec3f emission;
};

export void make_point_light(uniform const Light * uniform * uniform out, uniform const Vec3f * uniform position,
		uniform const Vec3f * uniform emission)
{
	uniform PointLight * uniform light = uniform new PointLight;
	light->light.type = POINT;
	light->position = *position;
	light->emission = *emission;
	*out = (uniform Light * uniform)light;
}
// Sample the light emitted from this point light incident on the position `pos`. The emission is the
// light's strength at pos, light dir is the direction to the light from `pos`
void point_light_incident(uniform const Light * uniform light, const Vec3f pos, Vec3f &emission,
		Vec3f &light_dir)
{
	uniform PointLight * uniform point = (uniform PointLight * uniform)light;
	light_dir = point->position - pos;
	emission = point->emission / dot(light_dir, light_dir);
	light_dir = normalize(light_dir);
}
export void drop_point_light(const uniform Light * uniform light){
	uniform PointLight * uniform point = (uniform PointLight * uniform)light;
	delete point;
}

enum MATERIAL_TYPE {
	LAMBERTIAN
};

struct Material {
	MATERIAL_TYPE type;
	Vec3f albedo;
};

struct Lambertian {
	Material mat;
};

export void make_lambertian(uniform const Material * uniform * uniform out, uniform const Vec3f * uniform albedo){
	uniform Lambertian * uniform lambertian = uniform new Lambertian;
	lambertian->mat.type = LAMBERTIAN;
	lambertian->mat.albedo = *albedo;
	*out = (uniform Material * uniform)lambertian;
}
Vec3f shade_lambertian(uniform const Material *mat, const Vec3f w_o, const Vec3f w_i){
	uniform Lambertian *lambertian = (uniform Lambertian *)mat;
	return lambertian->mat.albedo * M_1_PI;
}
export void drop_lambertian(uniform const Material * uniform mat){
	uniform Lambertian * uniform lambertian = (uniform Lambertian * uniform)mat;
	delete lambertian;
}

enum GEOM_TYPE {
	SPHERE,
	PLANE
};

struct Geometry {
	GEOM_TYPE type;
	const Material *material;
};

struct Sphere {
	Geometry geom;
	Vec3f center;
	float radius;
};

// An infinite plane centered at some point
struct Plane {
	Geometry geom;
	Vec3f center;
	Vec3f normal;
};

// Allocate and construct a new ISPC-side sphere, the pointer passed will be set to point to
// the newly allocated sphere
export void make_sphere(uniform const Geometry * uniform * uniform out, uniform const Vec3f * uniform center,
		uniform float radius, uniform const Material * uniform mat)
{
	uniform Sphere * uniform sphere = uniform new uniform Sphere;
	sphere->geom.type = SPHERE;
	sphere->geom.material = mat;
	sphere->center = *center;
	sphere->radius = radius;
	*out = (uniform Geometry * uniform)sphere;
}
// Release the ISPC-side sphere
export void drop_sphere(const uniform Geometry * uniform geom){
	uniform Sphere * uniform sphere = (uniform Sphere * uniform)geom;
	delete sphere;
}
inline bool sphere_intersect(Isect &isect, const Ray &ray, const uniform Geometry * uniform geom){
	uniform Sphere * uniform sphere = (uniform Sphere * uniform)geom;
	Vec3f rs = ray.origin - sphere->center;
	const float b = dot(rs, ray.dir);
	const float c = dot(rs, rs) - sphere->radius * sphere->radius;
	const float discrim = b * b - c;
	if (discrim > 0.f){
		float t = -b - sqrt(discrim);
		if (t <= 0.f || t >= isect.t){
			t = -b + sqrt(discrim);
		}
		if (t > 0.f && t < isect.t){
			isect.t = t;
			isect.p = ray.origin + ray.dir * t;
			isect.n = normalize(isect.p - sphere->center);
			return true;
		}
	}
	return false;
}

// Allocate and construct a new ISPC-side plane, the pointer passed will be set to point to
// the newly allocated sphere
export void make_plane(uniform const Geometry * uniform * uniform out, uniform const Vec3f * uniform center,
		uniform const Vec3f * uniform normal, uniform const Material * uniform mat)
{
	uniform Plane * uniform plane = uniform new uniform Plane;
	plane->geom.type = PLANE;
	plane->geom.material = mat;
	plane->center = *center;
	plane->normal = *normal;
	*out = (uniform Geometry * uniform)plane;
}
// Release the ISPC-side sphere
export void drop_plane(const uniform Geometry * uniform geom){
	uniform Plane * uniform plane = (uniform Plane * uniform)geom;
	delete plane;
}
inline bool plane_intersect(Isect &isect, const Ray &ray, const uniform Geometry * uniform geom){
	uniform Plane * uniform plane = (uniform Plane * uniform)geom;
	float t = dot(plane->center - ray.origin, plane->normal) / dot(ray.dir, plane->normal);
	if (t > 0.f && t < isect.t){
		isect.t = t;
		isect.p = ray.origin + ray.dir * t;
		isect.n = plane->normal;
		return true;
	}
	return false;
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
export void render(const uniform Camera * uniform camera, const uniform Geometry * const uniform * uniform geom,
		const uniform int32 n_geom, const uniform Light * uniform light, const uniform int32 seed,
		const uniform int32 width, const uniform int32 height, uniform float img[])
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
				intersect_scene(isect, ray, geom, n_geom);
				Vec3f ambient_light = make_vec3f(0.07, 0.07, 0.07);
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
					img[(j * width + i) * 3] += color.x;
					img[(j * width + i) * 3 + 1] += color.y;
					img[(j * width + i) * 3 + 2] += color.z;
				}
			}
			for (int c = 0; c < 3; ++c){
				img[(j * width + i) * 3 + c] *= inv_samples;
			}
		}
	}
}

