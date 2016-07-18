#include "vec3f.ih"
#include "camera.ih"

export void make_camera(uniform ISPCHandle* uniform out, const uniform Vec3f* uniform pos,
		const uniform Vec3f* uniform target, const uniform Vec3f* uniform up, const uniform float fovy,
		const uniform uint32_t width, const uniform uint32_t height) {
	uniform Camera* uniform camera = uniform new uniform Camera;
	camera->pos = to_short_vec(*pos);
	camera->up = to_short_vec(*up);
	camera->dir = to_short_vec(*target) - to_short_vec(*pos);
	const uniform float<3> dz = normalize(camera->dir);
	const uniform float<3> dx = normalize(cross(dz, camera->up));
	const uniform float<3> dy = normalize(cross(dx, dz));
	const uniform float dim_y = 2.0 * tan((fovy / 2.0) * PI / 180.0);
	const uniform float aspect_ratio = width / (float)height;
	const uniform float dim_x = dim_y * aspect_ratio;
	camera->screen_du = dx * dim_x;
	camera->screen_dv = dy * dim_y;
	camera->dir_top_left = dz - 0.5 * camera->screen_du - 0.5 * camera->screen_dv;
	camera->width = width;
	camera->height = height;
	*out = (uniform ISPCHandle* uniform)camera;
}
export void drop_camera(uniform ISPCHandle cam) {
	uniform Camera* uniform camera = (uniform Camera* uniform)cam;
	delete camera;
}

