(function() {var implementors = {};
implementors['rustc-serialize'] = ["<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='http://doc.rust-lang.org/nightly/core/fmt/struct.Error.html' title='core::fmt::Error'>Error</a>&gt; for <a class='enum' href='rustc-serialize/json/enum.EncoderError.html' title='rustc-serialize::json::EncoderError'>EncoderError</a>",];
implementors['hyper'] = ["<a class='stability Stable' title='Stable'></a>impl&lt;W&gt; <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='http://doc.rust-lang.org/nightly/std/io/buffered/struct.IntoInnerError.html' title='std::io::buffered::IntoInnerError'>IntoInnerError</a>&lt;W&gt;&gt; for <a class='struct' href='http://doc.rust-lang.org/nightly/std/io/error/struct.Error.html' title='std::io::error::Error'>Error</a>","<a class='stability Stable' title='Stable'></a>impl&lt;T&gt; <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='http://doc.rust-lang.org/nightly/std/sync/poison/struct.PoisonError.html' title='std::sync::poison::PoisonError'>PoisonError</a>&lt;T&gt;&gt; for <a class='enum' href='http://doc.rust-lang.org/nightly/std/sync/poison/enum.TryLockError.html' title='std::sync::poison::TryLockError'>TryLockError</a>&lt;T&gt;","<a class='stability Stable' title='Stable'></a>impl&lt;'a, E&gt; <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;E&gt; for <a class='struct' href='http://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html' title='alloc::boxed::Box'>Box</a>&lt;<a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.Error.html' title='core::error::Error'>Error</a> + 'a&gt;","<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='http://doc.rust-lang.org/nightly/std/old_io/struct.IoError.html' title='std::old_io::IoError'>IoError</a>&gt; for <a class='enum' href='hyper/enum.HttpError.html' title='hyper::HttpError'>HttpError</a>","<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='enum' href='url/parser/enum.ParseError.html' title='url::parser::ParseError'>ParseError</a>&gt; for <a class='enum' href='hyper/enum.HttpError.html' title='hyper::HttpError'>HttpError</a>",];
implementors['tweetust'] = ["<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='struct' href='tweetust/models/error/struct.ErrorResponse.html' title='tweetust::models::error::ErrorResponse'>ErrorResponse</a>&gt; for <a class='enum' href='tweetust/enum.TwitterError.html' title='tweetust::TwitterError'>TwitterError</a>","<a class='stability Stable' title='Stable'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/error/trait.FromError.html' title='core::error::FromError'>FromError</a>&lt;<a class='enum' href='hyper/enum.HttpError.html' title='hyper::HttpError'>HttpError</a>&gt; for <a class='enum' href='tweetust/enum.TwitterError.html' title='tweetust::TwitterError'>TwitterError</a>",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()