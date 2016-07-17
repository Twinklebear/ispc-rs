#include "vol.ih"

export void make_volume(uniform ISPCHandle* uniform out, const uniform Vec3i* uniform dims) {
	uniform Volume* uniform v = uniform new uniform Volume;
	v->dims = to_short_vec(*dims);
	// TODO: How to handle the data distributed version? allocating
	// the the whole thing up front is fine for a volume that fits in memory,
	// not so when it doesn't and is distributed
	v->data = uniform new uniform float[dims->x * dims->y * dims->z];
	v->value_range.x = INFINITY;
	v->value_range.y = -INFINITY;
	*out = v;
}
export void set_region(uniform ISPCHandle volume, const uniform float* uniform region,
		const uniform Vec3i* uniform start, const uniform Vec3i* uniform size) {
	uniform Volume* uniform v = (uniform Volume* uniform)volume;
	for (uniform int z = 0; z < size->z; ++z) {
		for (uniform int y = 0; y < size->y; ++y) {
			foreach (x = 0 ... size->x) {
				const float val = region[(z * size->y + y) * size->x + x];
				const int voxel = ((z + start->z) * v->dims.y + y + start->y) * v->dims.x
					+ x + start->x;
				v->data[voxel] = val;
				v->value_range.x = reduce_min(min(v->value_range.x, val));
				v->value_range.y = reduce_max(max(v->value_range.y, val));
			}
		}
	}
	print("volume value range % to %\n", v->value_range.x, v->value_range.y);
}
export void drop_volume(uniform ISPCHandle vol){
	uniform Volume* uniform v = (uniform Volume* uniform)vol;
	if (v->data != NULL){
		delete v->data;
	}
	delete v;
}
// Sample the volume at the position. pos should be within the normalized [0, 1] volume coordinates.
float sample_volume(const uniform Volume* uniform volume, const float<3> pos) {
	const float<3> tex_coords = pos * volume->dims;
	// TODO better than crap nearest neighbor sampling
	const int<3> nearest_neighbor = {
		clamp(tex_coords.x, 0.f, (float)volume->dims.x - 1),
		clamp(tex_coords.y, 0.f, (float)volume->dims.y - 1),
		clamp(tex_coords.z, 0.f, (float)volume->dims.z - 1)
	};
	const int voxel = (nearest_neighbor.z * volume->dims.y + nearest_neighbor.y) * volume->dims.x
		+ nearest_neighbor.x;
	return volume->data[voxel];
}
float<4> raycast_volume(const uniform ISPCHandle volume, Ray &ray){
	const uniform Volume* uniform vol = (uniform Volume* uniform)volume;
	const float<3> inv_dir = 1.0 / ray.dir;
	// Find the intersection with the box containing the volume [0, 1]
	const uniform float<3> box_max = {1, 1, 1};
	const uniform float<3> box_min = {0, 0, 0};
	const float<3> tmin_tmp = (box_min - ray.origin) * inv_dir;
	const float<3> tmax_tmp = (box_max - ray.origin) * inv_dir;
	const float<3> tmin = min(tmin_tmp, tmax_tmp);
	const float<3> tmax = max(tmin_tmp, tmax_tmp);
	const float tenter = max(0.f, max(tmin.x, max(tmin.y, tmin.z)));
	const float texit = min(tmax.x, min(tmax.y, tmax.z));

	float<4> color = {0, 0, 0, 0};
	if (tenter > texit){
		color.z = 1;
		return color;
	}

	// Now raymarch the volume
	const float<3> dt_vec = {
		rcp(vol->dims.x * abs(ray.dir.x)),
		rcp(vol->dims.y * abs(ray.dir.y)),
		rcp(vol->dims.z * abs(ray.dir.z))
	};
	const float dt = min(dt_vec.x, min(dt_vec.y, dt_vec.z));
	float<3> pos = ray.origin + tenter * ray.dir;
	for (float t = tenter; t < texit; t += dt){
		// TODO: Sample a transfer function!
		const float sample = (sample_volume(vol, pos) - vol->value_range.x)
			/ (vol->value_range.y - vol->value_range.x);
		const float<4> grayscale = {sample, sample, sample, sample};
		// TODO: Swizzles are read only!? I can't do color.xyz = ...?
		const float<3> col = (1.0 - color.w) * grayscale.w * grayscale.xyz;
		color.x += col.x;
		color.y += col.y;
		color.z += col.z;
		color.w += (1.0 - color.w) * grayscale.w;
		if (color.w >= 0.98) {
			break;
		}
		pos += dt * ray.dir;
	}
	return color;
}

