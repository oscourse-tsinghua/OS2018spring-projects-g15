macro_rules! test_end {
	() => (
		println!("Test end");
		// test success	
		unsafe{ arch::cpu::exit_in_qemu(11) }
	)
}

macro_rules! test {
	($func:ident) => (
		if cfg!(feature = "test") {
			println!("Testing: {}", stringify!($func));
			use self::test::$func;
			$func();
			println!("Success: {}", stringify!($func));
		}
	)
}