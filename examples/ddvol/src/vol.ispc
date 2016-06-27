#include "vol.ih"

export void make_volume(uniform ISPCHandle* uniform out, const uniform uint64 width,
		const uniform uint64 height, const uniform uint64 depth, const uniform DataType data_type){
	uniform Volume* uniform v = uniform new Volume;
	*out = v->handle;
}
export void drop_volume(uniform ISPCHandle vol){
	uniform Volume* uniform v = (uniform Volume* uniform)vol;
	delete v;
}

