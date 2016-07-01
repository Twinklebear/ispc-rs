#include "vol.ih"

static inline uniform unsigned int64 dtype_size(const uniform DataType dtype){
	switch (dtype) {
		case UCHAR: return 1;
		case USHORT: return 2;
		case FLOAT: return 4;
	}
}

export void make_volume(uniform ISPCHandle* uniform out, const uniform uint64 width,
		const uniform uint64 height, const uniform uint64 depth, const uniform DataType data_type){
	uniform Volume* uniform v = uniform new Volume;
	v->data = NULL;
	v->dims[0] = width;
	v->dims[1] = height;
	v->dims[2] = depth;
	v->data_type = data_type;
	// We don't allocate anything until set region starts being called
	// So really would we want a bricked volume? How could we manage having
	// disjoint chunks of data on some node? I guess we **must** use a brick
	// based volume renderer since we'll have some bricks of volume data assigned
	// to each node
	v->data = NULL;
	*out = v->handle;
}
export void drop_volume(uniform ISPCHandle vol){
	uniform Volume* uniform v = (uniform Volume* uniform)vol;
	if (v->data != NULL){
		delete v->data;
	}
	delete v;
}

