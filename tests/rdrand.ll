declare {i16, i32} @llvm.x86.rdrand.16()
declare {i32, i32} @llvm.x86.rdrand.32()
declare {i64, i32} @llvm.x86.rdrand.64()

define i64 @librdrand_rust_rand_64() {
    br label %body
body:
    %result = tail call {i64, i32} @llvm.x86.rdrand.64() nounwind
    %flag = extractvalue {i64, i32} %result, 1
    %boolflag = icmp eq i32 %flag, 0
    br i1 %boolflag, label %body, label %done
done:
    %val = extractvalue {i64, i32} %result, 0
    ret i64 %val
}

define i32 @librdrand_rust_rand_32() {
    br label %body
body:
    %result = tail call {i32, i32} @llvm.x86.rdrand.32() nounwind
    %flag = extractvalue {i32, i32} %result, 1
    %boolflag = icmp eq i32 %flag, 0
    br i1 %boolflag, label %body, label %done
done:
    %val = extractvalue {i32, i32} %result, 0
    ret i32 %val
}

define i16 @librdrand_rust_rand_16() {
    br label %body
body:
    %result = tail call {i16, i32} @llvm.x86.rdrand.16() nounwind
    %flag = extractvalue {i16, i32} %result, 1
    %boolflag = icmp eq i32 %flag, 0
    br i1 %boolflag, label %body, label %done
done:
    %val = extractvalue {i16, i32} %result, 0
    ret i16 %val
}
