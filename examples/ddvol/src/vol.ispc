#include "vol.ih"

export void make_volume(uniform ISPCHandle* uniform out, const uniform Vec3i* uniform dims) {
	uniform Volume* uniform v = uniform new Volume;
	v->dims = to_short_vec(*dims);
	// TODO: How to handle the data distributed version? allocating
	// the the whole thing up front is fine for a volume that fits in memory,
	// not so when it doesn't and is distributed
	v->data = uniform new uniform float[dims->x * dims->y * dims->z];
	*out = v->handle;
}
export void set_region(uniform ISPCHandle* uniform volume, const float* uniform region,
		const uniform Vec3i* uniform _start, const uniform Vec3i* uniform _size) {
	uniform Volume* uniform v = (uniform Volume* uniform)volume;
	const uniform int<3> start = to_short_vec(*_start);
	const uniform int<3> size = to_short_vec(*_size);
	const uniform int<3> end = start + size;
	for (uniform int z = start.z; z < end.z; ++z) {
		for (uniform int y = start.y; y < end.y; ++y) {
			foreach (x = start.x ... end.x) {
				v->data[(z * v->dims.y + y) * v->dims.x + x] = region[(z * size.y + y) * size.x + x];
			}
		}
	}
}
export void drop_volume(uniform ISPCHandle vol){
	uniform Volume* uniform v = (uniform Volume* uniform)vol;
	if (v->data != NULL){
		delete v->data;
	}
	delete v;
}

