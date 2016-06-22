#include "material.ih"

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
float pdf_lambertian(uniform const Material *mat, const Vec3f w_o, const Vec3f w_i, const Vec3f n){
	if (same_hemisphere(w_o, w_i, n)){
		return abs(cos(dot(w_o, n))) * M_1_PI;
	} else {
		return 0.0;
	}
}
export void drop_lambertian(uniform const Material * uniform mat){
	uniform Lambertian * uniform lambertian = (uniform Lambertian * uniform)mat;
	delete lambertian;
}

