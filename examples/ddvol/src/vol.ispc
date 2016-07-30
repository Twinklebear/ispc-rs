#include "vol.ih"

export void make_volume(uniform ISPCHandle* uniform out, const uniform Vec3i* uniform dims,
						uniform ISPCHandle tfn)
{
	uniform Volume* uniform v = uniform new uniform Volume;
	v->dims = to_short_vec(*dims);
	// Find scale factor for each axis such that the longest axis is [0, 1] 
	const uniform float max_axis = max(v->dims.x, max(v->dims.y, v->dims.z));
	v->box_max = v->dims / max_axis;
	// TODO: How to handle the data distributed version? allocating
	// the the whole thing up front is fine for a volume that fits in memory,
	// not so when it doesn't and is distributed
	v->data = uniform new uniform float[dims->x * dims->y * dims->z];
	v->value_range.x = INFINITY;
	v->value_range.y = -INFINITY;
	v->tfn = tfn;
	v->isovalue = -1.0;
	*out = v;
}
export void volume_set_transfer_function(uniform ISPCHandle volume, uniform ISPCHandle tfn){
	uniform Volume* uniform v = (uniform Volume* uniform)volume;
	v->tfn = tfn;
}
export void volume_set_isovalue(uniform ISPCHandle volume, const uniform float isovalue){
	uniform Volume* uniform v = (uniform Volume* uniform)volume;
	v->isovalue = (isovalue - v->value_range.x) / (v->value_range.y - v->value_range.x);
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
}
export void drop_volume(uniform ISPCHandle vol){
	uniform Volume* uniform v = (uniform Volume* uniform)vol;
	if (v->data != NULL){
		delete[] v->data;
	}
	delete v;
}
static inline float get_voxel(const uniform Volume* uniform v, const int x, const int y, const int z) {
	const int tx = clamp(x, 0, v->dims.x - 1);
	const int ty = clamp(y, 0, v->dims.y - 1);
	const int tz = clamp(z, 0, v->dims.z - 1);
	return v->data[(tz * v->dims.y + ty) * v->dims.y + tx];
}
// Perform bilinear sampling on a slice of the volume data
static inline float bilinear_sample(const uniform Volume* uniform v, const float<2> coords, const int z){
	const int<2> origin = {floor(coords.x), floor(coords.y)};
	const float<2> local = coords - origin;
	return get_voxel(v, origin.x, origin.y, z) * (1.0 - local.x) * (1.0 - local.y)
		+ get_voxel(v, origin.x + 1, origin.y, z) * local.x * (1.0 - local.y)
		+ get_voxel(v, origin.x, origin.y + 1, z) * (1.0 - local.x) * local.y
		+ get_voxel(v, origin.x + 1, origin.y + 1, z) * local.x * local.y;
}
// Sample the volume at the position. pos should be within the normalized [0, 1] volume coordinates.
static inline float sample_volume(const uniform Volume* uniform volume, const float<3> pos){
	const float<3> tex_coords = pos * volume->dims;
	const int lo_z = floor(tex_coords.z);
	const float lo_slice = bilinear_sample(volume, tex_coords.xy, lo_z);
	const float hi_slice = bilinear_sample(volume, tex_coords.xy, lo_z + 1);
	const float t_val = tex_coords.z - lo_z;
	return (1.0 - t_val) * lo_slice + t_val * hi_slice;
}
static inline float<3> sample_volume_gradient(const uniform Volume* uniform volume,
		const float<3> pos, const float dt)
{
	const float<2> h = {dt, 0};
	const float<3> grad = {
		sample_volume(volume, pos + h.xyy) - sample_volume(volume, pos - h.xyy),
		sample_volume(volume, pos + h.yxy) - sample_volume(volume, pos - h.yxy),
		sample_volume(volume, pos + h.yyx) - sample_volume(volume, pos - h.yyx)
	};
	return grad / (2.0 * dt);
}
float<4> raycast_volume(const uniform ISPCHandle volume, Ray &ray){
	const uniform Volume* uniform vol = (uniform Volume* uniform)volume;
	const float<3> inv_dir = 1.0 / ray.dir;
	// Find the intersection with the box containing the volume [0, 1]
	const uniform float<3> box_min = {0, 0, 0};
	const float<3> tmin_tmp = (box_min - ray.origin) * inv_dir;
	const float<3> tmax_tmp = (vol->box_max - ray.origin) * inv_dir;
	const float<3> tmin = min(tmin_tmp, tmax_tmp);
	const float<3> tmax = max(tmin_tmp, tmax_tmp);
	const float tenter = max(0.f, max(tmin.x, max(tmin.y, tmin.z)));
	const float texit = min(tmax.x, min(tmax.y, tmax.z));

	float<4> color = {0, 0, 0, 0};
	if (tenter > texit){
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
	const uniform float isovalue = vol->isovalue;
	const uniform bool isosurface = isovalue >= 0.0;
	float prev_val = 0.f;
	for (float t = tenter; t < texit; t += dt){
		const float value = (sample_volume(vol, pos) - vol->value_range.x)
			/ (vol->value_range.y - vol->value_range.x);
		float<4> tfn_sample = sample_transfer_function(vol->tfn, value);
		float<3> col;
		if (isosurface) {
			if (t != tenter && prev_val <= isovalue && value >= isovalue) {
				const float<3> prev_pos = pos - dt * ray.dir;
				pos = (isovalue - prev_val) / (value - prev_val) * (pos - prev_pos) + prev_pos;
				tfn_sample.w = 1.f;
				const float<3> normal = -normalize(sample_volume_gradient(vol, pos, dt));
				const float lighting = clamp(dot(normal, -ray.dir), 0.1, 1.0);
				tfn_sample.x *= lighting;
				tfn_sample.y *= lighting;
				tfn_sample.z *= lighting;
			}
			prev_val = value;
		}
		col = (1.0 - color.w) * tfn_sample.w * tfn_sample.xyz;
		// TODO: I can't do color.xyz += ...?
		color.x += col.x;
		color.y += col.y;
		color.z += col.z;
		color.w += (1.0 - color.w) * tfn_sample.w;
		if (color.w >= 0.98) {
			break;
		}
		pos += dt * ray.dir;
	}
	return color;
}

