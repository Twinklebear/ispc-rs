# rt

This example is a simple pathtracer supporting planes and spheres with Lambertian materials
illuminated by diffuse light sources. It shows more complicated interoperability with Rust code
where we have corresponding Rust and ISPC side structs working together in the program.

Note that here for dynamic dispatch I'm using enums to tag the types of the geometry, lights
and materials since the structs are shared with Rust and it gets a bit ugly w/ ISPC's exported
function pointers. Another option is to have the `ispc_equiv` pointers be just void pointers
so the type and function pointers and such don't need to be bound in Rust and are more opaque.
