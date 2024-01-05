pub mod macros {
    /// Takes a number and a mutable array of bytes
    /// and writes to the array of bytes a u8 bytes
    /// representation of the number
    /// # Example
    /// ```ignore
    /// let mut bytes = [0; 2];
    /// usize_to_u8_bytes!(7000, mut bytes);
    /// assert_eq!(bytes, [27, 88]);
    /// ```
    /// # Panic
    /// Bytes must be at least length 2 or more
    macro_rules! usize_to_u8_bytes {
        ($num: expr, mut $bytes: expr) => {
            if $bytes.len() < 2 {
                panic!("Bytes passes must be at least length 2 or more");
            }

            let number = usize::from($num);
            let max_number = usize::pow(2, 8);

            let bytes_length = $bytes.len() - 1;

            for i in 0..bytes_length {
                let i_num = ((number as f64 / (usize::pow(max_number, (i + 1) as u32) as f64))
                    .floor() as usize
                    % max_number) as u8;
                $bytes[i] = i_num;
            }

            $bytes[bytes_length] = (number % max_number) as u8;
        };
        ($num: expr; $bytes_len: expr) => {{
            let mut bytes = [0; $bytes_len];
            usize_to_u8_bytes!($num, mut bytes);
            bytes
        }};
    }

    /// Takes bytes and converts them to a number
    /// # Example
    ///  ```ignore
    /// let mut bytes = vec![27, 88];
    /// let number = u8_bytes_to_usize!(bytes);
    /// assert_eq!(number, 7000);
    /// ```
    /// # Panic
    /// Bytes must be at least length 2 or more
    macro_rules! u8_bytes_to_usize {
        ( $bytes: expr ) => {{
            if $bytes.len() < 2 {
                panic!("Bytes passes must be at least length 2 or more");
            }

            let max_number = usize::pow(2, 8);
            let bytes_length = $bytes.len() - 1;
            let mut count = 0;

            for i in 0..bytes_length {
                count += $bytes[i] as usize * usize::pow(max_number, (i + 1) as u32);
            }
            count += $bytes[bytes_length] as usize;
            count
        }};
    }

    pub(crate) use u8_bytes_to_usize;
    pub(crate) use usize_to_u8_bytes;
}

#[cfg(test)]
mod tests {
    use crate::utils::macros::{u8_bytes_to_usize, usize_to_u8_bytes};

    #[derive(Debug)]
    struct Pair {
        pub number: usize,
        pub bytes: Vec<u8>,
    }
    impl Pair {
        pub fn new(number: usize, bytes: Vec<u8>) -> Self {
            Self { number, bytes }
        }
    }

    #[test]
    fn conversion_to_bytes() {
        let tests: Vec<Pair> = vec![
            Pair::new(6598656 as usize, vec![176, 100, 0, 0]),
            Pair::new(9458456 as usize, vec![83, 144, 0, 24]),
            Pair::new(3904954 as usize, vec![149, 59, 0, 186]),
        ];

        for test in tests {
            let bytes = usize_to_u8_bytes!((test.number); 4);
            assert_eq!(bytes.to_vec(), test.bytes);
        }
    }

    #[test]
    fn conversion_to_number() {
        let tests: Vec<Pair> = vec![
            Pair::new(6598656 as usize, vec![176, 100, 0, 0]),
            Pair::new(9458456 as usize, vec![83, 144, 0, 24]),
            Pair::new(3904954 as usize, vec![149, 59, 0, 186]),
        ];

        
        for test in tests {
            let bytes = usize_to_u8_bytes!((test.number); 4);
            assert_eq!(bytes.to_vec(), test.bytes);
            assert_eq!(u8_bytes_to_usize!(bytes), test.number);
        }
    }

    #[test]
    fn big_numbers() {
        let tests: Vec<Pair> = vec![
            Pair::new(usize::pow(2, 16) - 1, vec![255, 0, 0, 255]),
            Pair::new(usize::pow(2, 24) - 1, vec![255, 255, 0, 255]),
            Pair::new(usize::pow(2, 32) - 1, vec![255, 255, 255, 255]),
        ];

        for test in tests {
            let bytes = usize_to_u8_bytes!((test.number); 4);
            assert_eq!(bytes.to_vec(), test.bytes);
            assert_eq!(u8_bytes_to_usize!(bytes), test.number);
        }
    }
}
