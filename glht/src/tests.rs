use super::*;

use super::v2::*;

#[test]
fn test_scr_box() {

    use textscreen::ScrBox;

    let in_box = V2{x:5,y:5};
    let outside_edge_box = V2{x:10,y:5};
    let on_edge_box = V2{x:9,y:5};
    let outside_box = V2{x:-1,y:-1};

    let big_box = ScrBox::new(V2{x:-1,y:-1}, V2{x:11,y:21});

    let br = big_box.get_br();
    assert_eq!(br, V2::new(9,19));

    assert!(big_box.in_bounds(&in_box));
    assert!(big_box.in_bounds(&on_edge_box));
    assert_eq!(big_box.in_bounds(&outside_edge_box), false);
    assert_eq!(big_box.in_bounds(&outside_box), true);

    let big_to_clip = ScrBox::new(V2{x:-100,y:-100}, V2{x:500,y:520});

    let clipped_box = ScrBox::clip_box(&big_box, &big_to_clip);

    assert_eq!(Some(big_box.clone()), clipped_box);


    let small_to_clip = ScrBox::new(V2{x:1,y:2}, V2{x:3,y:4});
    let clipped_box = ScrBox::clip_box(&big_box, &small_to_clip);
    assert_eq!(Some(small_to_clip), clipped_box);
}
