# Figure out what to do about eg hemispheres

Hemispheres cause problems, because usually the way it would be done would be to have all of the pixels in the leftmost column be at the same point on one pole, and all of the pixels in the rightmost column be at the opposite pole. That doesn't currently work because we don't allow adjacent pixels to map to the same point (since a triangle with two identical points isn't valid).

### Test case (it's inside out and not correctly oriented, the correct equations can be determined once it actually works)

**X:** sin(pi-pi*y/(h-1))*cos(pi*x/(w - 1))*(w - 1)/pi/5
**Y:** sin(pi-pi*y/(h-1))*sin(pi*x/(w - 1))*(w - 1)/pi/5
**Z:** cos(pi-pi*y/(h-1))*(h-1)/pi/5

### Possible solutions

#### Provide an option to reduce an edge to a single point

This would probably be a per-edge option that would average the pixel values and make a single point that is considered adjacent to every point for the next inside row/column. For instance, condensing the left edge to a single point would average all pixel values from the leftmost column and use that to create a single point (probably using x=0, y=0 for the equations), then create triangles from that point to every sequential pair of points from the column x=1.

Even better, we could detect at runtime if every point for an edge is identical (or close beyond some very small limit to prevent issues with float accuracy). If some but not all points for an edge are identical, it would still fail with a message explaining to the user what's wrong.


# Add f32 support to meval

I'm not sure how much performance we lose with all the f64 <-> f32 conversion, but adding f32 support to meval would certainly be an improvement.
