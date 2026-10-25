//! The long-line rule.

/// Members that test the long-line rule.
pub struct Wide {
    /// This doc comment line is deliberately longer than eighty columns so it shows comments never wrap.
    pub a_field_with_a_very_long_type:
        std::collections::BTreeMap<String, std::collections::BTreeSet<u64>>,
}

impl Wide {
    pub fn exactly_at_80(first_value: u64, second_value: u64) -> Option<String> {
        None
    }

    pub fn one_over_limit(first_value: u64, second_value: u64) -> Option<String> {
        None
    }

    pub fn long_first_line(argument_number_one: u64, argument_number_two: u64) -> u64 {
        0
    }

    pub fn tail_also_breaks(
        argument_number_one: u64,
        argument_number_two: u64,
    ) -> std::collections::BTreeMap<String, Vec<u64>> {
        todo!()
    }

    pub fn register(
        &mut self,
        name_of_the_thing: &str,
        handler_for_the_thing: Box<dyn Fn(u32)>,
        priority: u8,
    ) {
    }

    pub fn register_bounded<T>(&mut self, name: &str, value: T)
    where
        T: Clone + Send + Sync + std::fmt::Debug + std::hash::Hash,
    {
    }

    pub fn huge(
        an_extremely_long_parameter_name_for_testing: std::collections::BTreeMap<String, Vec<u64>>,
    ) {
    }

    pub fn fits_with_comment(value: u64) -> u64 { value } // a trailing comment that pushes past eighty
}

/// A header whose where clause moves.
pub struct Container<K, V>
where
    K: Ord + Clone + std::fmt::Debug + Send + Sync,
    V: Clone + Send,
{
    keys: Vec<K>,
    values: Vec<V>,
}
