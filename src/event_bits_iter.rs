use {
    std::{
        borrow::{
            Cow
        },
        iter::{
            FusedIterator
        },
        marker::{
            PhantomData
        }
    }
};

pub(crate) struct EventBitsIter< 'a, T > {
    buffer: Cow< 'a, [u8] >,
    index: usize,
    bit: usize,
    byte: u8,
    phantom: PhantomData< T >
}

impl< 'a, T > EventBitsIter< 'a, T > {
    pub(crate) fn new( buffer: Cow< 'a, [u8] > ) -> Self {
        let byte = buffer.first().cloned().unwrap_or( 0 );
        EventBitsIter {
            buffer,
            index: 0,
            bit: 0,
            byte,
            phantom: PhantomData
        }
    }
}

impl< 'a, T > Iterator for EventBitsIter< 'a, T > where T: From< u16 > {
    type Item = T;
    fn next( &mut self ) -> Option< Self::Item > {
        while self.byte == 0 {
            if self.index + 1 >= self.buffer.len() {
                return None;
            }

            self.index += 1;
            self.byte = self.buffer[ self.index ];
            self.bit = 0;
        }

        while self.byte & 1 == 0 {
            self.byte >>= 1;
            self.bit += 1;
        }

        let output = self.index * 8 + self.bit;
        self.byte >>= 1;
        self.bit += 1;

        Some( (output as u16).into() )
    }
}

impl< 'a, T > FusedIterator for EventBitsIter< 'a, T > where T: From< u16 > {}

#[test]
fn test_event_bits_iter_empty() {
    let mut iter = EventBitsIter::< u16 >::new( &[] );
    assert_eq!( iter.next(), None );
}

#[test]
fn test_event_bits_iter_single_element_empty() {
    let mut iter = EventBitsIter::< u16 >::new( &[0] );
    assert_eq!( iter.next(), None );
}

#[test]
fn test_event_bits_iter_multiple_elements_empty() {
    let mut iter = EventBitsIter::< u16 >::new( &[0, 0, 0, 0] );
    assert_eq!( iter.next(), None );
}

#[test]
fn test_event_bits_iter_single_element_first_bit() {
    let mut iter = EventBitsIter::< u16 >::new( &[0b0000_0001] );
    assert_eq!( iter.next(), Some( 0 ) );
    assert_eq!( iter.next(), None );
}

#[test]
fn test_event_bits_iter_single_element_last_bit() {
    let mut iter = EventBitsIter::< u16 >::new( &[0b1000_0000] );
    assert_eq!( iter.next(), Some( 7 ) );
    assert_eq!( iter.next(), None );
}

#[test]
fn test_event_bits_iter_single_element_multiple_bits() {
    let mut iter = EventBitsIter::< u16 >::new( &[0b1000_0001] );
    assert_eq!( iter.next(), Some( 0 ) );
    assert_eq!( iter.next(), Some( 7 ) );
    assert_eq!( iter.next(), None );
}

#[test]
fn test_event_bits_iter_single_element_multiple_elements() {
    let mut iter = EventBitsIter::< u16 >::new( &[0, 0b1000_0001, 0] );
    assert_eq!( iter.next(), Some( 8 ) );
    assert_eq!( iter.next(), Some( 15 ) );
    assert_eq!( iter.next(), None );
}
