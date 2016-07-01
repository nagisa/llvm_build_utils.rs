define i128 @test(i64 %a, i64 %b) {
    %x = zext i64 %a to i128
    %y = zext i64 %b to i128
    %r = mul nuw i128 %x, %y
    ret i128 %r
}
