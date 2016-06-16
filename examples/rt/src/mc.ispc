#include "mc.ih"

Vec3f cos_sample_hemisphere(const float s0, const float s1){
	// We use Malley's method here, generate samples on a disk then project them up to the hemisphere
	float u, v;
	concentric_sample_disk(s0, s1, u, v);
	return make_vec3f(u, v, sqrt(max(0.f, 1.f - u * u - v * v)));
}
void concentric_sample_disk(const float u0, const float u1, float &u, float &v){
	const float s0 = 2.0 * u0 - 1.0;
	const float s1 = 2.0 * u1 - 1.0;
	float radius;
	float theta;
	if (s0 == 0.0 && s1 == 0.0){
		u = 0.0;
		v = 0.0;
		return;
	}
	if (s0 >= -s1){
		if (s0 > s1){
			radius = s0;
			if (s1 > 0.0){
				theta = s1 / s0;
			} else {
				theta = 8.0 + s1 / s0;
			}
		}
		else {
			radius = s1;
			theta = 2.0 - s0 / s1;
		}
	}
	else {
		if (s0 <= s1){
			radius = -s0;
			theta = 4.0 + s1 / s0;
		}
		else {
			radius = -s1;
			theta = 6.0 - s0 / s1;
		}
	}
	theta = theta * M_PI_4;
	u = radius * cos(theta);
	v = radius * sin(theta);
}

