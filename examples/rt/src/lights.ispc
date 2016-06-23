#include "lights.ih"

export void make_point_light(const void** uniform out, uniform const Vec3f * uniform position,
		uniform const Vec3f * uniform emission)
{
	uniform PointLight * uniform light = uniform new PointLight;
	light->light.incident = point_light_incident;
	light->light.occlusion_tester = point_light_occlusion_test;

	light->position = *position;
	light->emission = *emission;
	*out = (void*)light;
}
void point_light_incident(uniform const Light * uniform light, const Vec3f pos, Vec3f &emission,
		Vec3f &light_dir)
{
	uniform PointLight * uniform point = (uniform PointLight * uniform)light;
	light_dir = point->position - pos;
	emission = point->emission / dot(light_dir, light_dir);
	light_dir = normalize(light_dir);
}
void point_light_occlusion_test(uniform const Light * uniform light, Ray &ray, Isect &test){
	uniform PointLight * uniform point = (uniform PointLight * uniform)light;
	Vec3f light_dir = point->position - ray.origin;
	ray.dir = light_dir;
	test.hit = 0;
	// We shouldn't find hits past the light
	test.t = 0.999;
}
export void drop_point_light(const void* uniform light){
	uniform PointLight * uniform point = (uniform PointLight * uniform)light;
	delete point;
}


