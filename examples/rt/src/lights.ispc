#include "lights.ih"

export void make_point_light(uniform const Light * uniform * uniform out, uniform const Vec3f * uniform position,
		uniform const Vec3f * uniform emission)
{
	uniform PointLight * uniform light = uniform new PointLight;
	light->light.type = POINT;
	light->position = *position;
	light->emission = *emission;
	*out = (uniform Light * uniform)light;
}
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


