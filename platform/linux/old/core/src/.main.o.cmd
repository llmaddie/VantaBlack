savedcmd_main.o := OBJTREE=/home/mad/linux RUST_MODFILE=./vantablack rustc --edition=2021 -Zbinary_dep_depinfo=y -Astable_features -Aunused_features -Dnon_ascii_idents -Dunsafe_op_in_unsafe_fn -Wmissing_docs -Wrust_2018_idioms -Wunreachable_pub -Wclippy::all -Wclippy::as_ptr_cast_mut -Wclippy::as_underscore -Wclippy::cast_lossless -Aclippy::collapsible_if -Aclippy::collapsible_match -Wclippy::ignored_unit_patterns -Aclippy::incompatible_msrv -Wclippy::mut_mut -Wclippy::needless_bitwise_bool -Aclippy::needless_lifetimes -Wclippy::no_mangle_with_rust_abi -Wclippy::ptr_as_ptr -Wclippy::ptr_cast_constness -Wclippy::ref_as_ptr -Wclippy::undocumented_unsafe_blocks -Aclippy::uninlined_format_args -Wclippy::unnecessary_safety_comment -Wclippy::unnecessary_safety_doc -Wrustdoc::missing_crate_level_docs -Wrustdoc::unescaped_backticks -Cpanic=abort -Cembed-bitcode=n -Clto=n -Cforce-unwind-tables=n -Ccodegen-units=1 -Csymbol-mangling-version=v0 -Crelocation-model=static -Zfunction-sections=n -Wclippy::float_arithmetic --target=/home/mad/linux/scripts/target.json -Ctarget-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2 -Zcf-protection=branch -Cjump-tables=n -Ctarget-cpu=x86-64 -Ztune-cpu=generic -Cno-redzone=y -Ccode-model=kernel -Zfunction-return=thunk-extern -Zpatchable-function-entry=16,16 -Copt-level=2 -Cdebug-assertions=n -Coverflow-checks=y -Dwarnings  --cfg MODULE  @/home/mad/linux/include/generated/rustc_cfg -Zallow-features=arbitrary_self_types,asm_goto,generic_arg_infer,used_with_arg -Zcrate-attr=no_std -Zcrate-attr='feature(arbitrary_self_types,asm_goto,generic_arg_infer,used_with_arg)' -Zunstable-options --extern pin_init --extern kernel --crate-type rlib -L /home/mad/linux/rust/ --sysroot=/dev/null --out-dir ./ --emit=dep-info=./.main.o.d --emit=obj=main.o main.rs  

source_main.o := main.rs

deps_main.o := \
  /home/mad/linux/rust/libcore.rmeta \
  /home/mad/linux/rust/libkernel.rmeta \
  /home/mad/linux/rust/libffi.rmeta \
  /home/mad/linux/rust/libcompiler_builtins.rmeta \
  /home/mad/linux/rust/libpin_init.rmeta \
  /home/mad/linux/rust/libpin_init_internal.so \
  /home/mad/linux/rust/libmacros.so \
  /home/mad/linux/rust/libbuild_error.rmeta \
  /home/mad/linux/rust/libbindings.rmeta \
  /home/mad/linux/rust/libuapi.rmeta \

main.o: $(deps_main.o)

$(deps_main.o):
