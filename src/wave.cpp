#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <sstream>
#include <cstdlib>

// Static wave configuration
#define BOOST_WAVE_SUPPORT_CPP1Z 1
#define BOOST_WAVE_SUPPORT_MS_EXTENSIONS 0
#define BOOST_WAVE_PRAGMA_KEYWORD "mtsc"
#define BOOST_WAVE_SUPPORT_PRAGMA_MESSAGE 0

#include <boost/wave.hpp>
#include <boost/wave/cpplexer/cpp_lex_token.hpp>
#include <boost/wave/cpplexer/cpp_lex_iterator.hpp>
#include <boost/regex.hpp>

#define UFFFF "\uffff"
#define UFFFE "\ufffe"

#include "wave.h"
using namespace wave;

typedef std::function<void(const MessageType TYPE, std::string filename, const i32 LINE, const std::string MESSAGE)> message_callback;

const std::string HASHBANG_PREFIX = "#!";

template<typename TokenT>
class wave_hooks : public boost::wave::context_policies::eat_whitespace<TokenT>
{
    typedef boost::wave::context_policies::eat_whitespace<TokenT> base_type;
    private:
        const bool PRESERVE_WHITESPACE;       // enable whitespace preservation
        const bool PRESERVE_BOL_WHITESPACE;   // enable beginning of line whitespace preservation
        message_callback on_message;
        const boost::wave::util::file_position_type* const P_CURRENT_POSITION;

    public:
        wave_hooks(const bool PRESERVE_WHITESPACE, const bool PRESERVE_BOL_WHITESPACE, message_callback on_message, const boost::wave::util::file_position_type* const P_CURRENT_POSITION) : PRESERVE_WHITESPACE(PRESERVE_WHITESPACE), PRESERVE_BOL_WHITESPACE(PRESERVE_BOL_WHITESPACE), on_message(on_message), P_CURRENT_POSITION(P_CURRENT_POSITION) {}

        template<typename ContextT>
        bool may_skip_whitespace(ContextT const &ctx, TokenT &token, bool &skipped_newline) {
            return this->base_type::may_skip_whitespace(
                    ctx, token, need_preserve_comments(ctx.get_language()),
                    PRESERVE_BOL_WHITESPACE, skipped_newline) ?
                !PRESERVE_WHITESPACE : false;
        }

        template <typename ContextT, typename ContainerT>
        bool found_warning_directive(ContextT const& ctx, ContainerT const& message) {
            on_message(MessageType::WARNING, P_CURRENT_POSITION->get_file().c_str(), P_CURRENT_POSITION->get_line(), boost::wave::util::impl::as_string(message).c_str());
            return true;
        }

        template <typename ContextT, typename ContainerT>
        bool found_error_directive(ContextT const& ctx, ContainerT const& message) {
            on_message(MessageType::ERROR, P_CURRENT_POSITION->get_file().c_str(), P_CURRENT_POSITION->get_line(), boost::wave::util::impl::as_string(message).c_str());
            return true;
        }
};

const boost::regex COMMENT_MODE_INPUT_ADJUSTMENT_PATTERN(R"XXX(^(\s*?)\/\/\/(?=\s*#))XXX"),
                   COMMENT_MODE_OUTPUT_ADJUSTMENT_PATTERN(R"(\r?\n)" UFFFF R"(\r?\n(?=\s*#))"),
                   COMMENT_MODE_OUTPUT_ADJUSTMENT_PATTERN_EMPTY(R"(\r?\n)" UFFFF R"(\r?\n)"),
                   LINE_CONTINUATION_PATTERN(R"(\\\r?\n)"),
                   LINE_CONTINUATION_UNDO_PATTERN(UFFFE)
    ;

std::string& apply_input_adjustment(std::string &text, const bool ADD_NEWLINE = true, const bool DISCARD_HASHBANG = false) {
    if(DISCARD_HASHBANG && std::equal(HASHBANG_PREFIX.begin(), HASHBANG_PREFIX.end(), text.begin())) {
        text = text.substr(text.find("\n")); // Leaves line numbers unchanged
    }
    
    return text = boost::regex_replace(
        boost::regex_replace(
            boost::regex_replace(text,
                LINE_CONTINUATION_PATTERN, UFFFE
            ),
            COMMENT_MODE_INPUT_ADJUSTMENT_PATTERN, "$1" "\n" UFFFF "\n"
        ),
        LINE_CONTINUATION_UNDO_PATTERN, "\\\\\n"
    ) + (ADD_NEWLINE ? "\n" : ""); // Add an extra \n to the end; wave fails on a trailing comment
}

std::string& apply_output_adjustment(std::string &text) {
    return text = boost::regex_replace(
        boost::regex_replace(text,
            COMMENT_MODE_OUTPUT_ADJUSTMENT_PATTERN, "///"
        ),
        COMMENT_MODE_OUTPUT_ADJUSTMENT_PATTERN_EMPTY, ""
    );
}

struct adjusted_input_policy {
    template<typename IterContextT>
    class inner {
        public:
            template<typename PositionT>
            static void init_iterators(IterContextT &iter_ctx, PositionT const &act_pos, boost::wave::language_support language) {
                typedef typename IterContextT::iterator_type iterator_type;

                boost::filesystem::ifstream instream(iter_ctx.filename.c_str());
                if (!instream.is_open()) {
                    BOOST_WAVE_THROW_CTX(iter_ctx.ctx, boost::wave::preprocess_exception,
                        bad_include_file, iter_ctx.filename.c_str(), act_pos);
                    return;
                }
                instream.unsetf(std::ios::skipws);

                iter_ctx.instring.assign(
                    std::istreambuf_iterator<char>(instream.rdbuf()),
                    std::istreambuf_iterator<char>());

                apply_input_adjustment(iter_ctx.instring, false, true);

                iter_ctx.first = iterator_type(
                    iter_ctx.instring.begin(), iter_ctx.instring.end(),
                    PositionT(iter_ctx.filename), language);
                iter_ctx.last = iterator_type();
            }

        private:
            std::string instring;
    };
};

#include <stdexcept>
#include <exception>
#include <cxxabi.h>
const char* get_current_exception_name() {
    int status;
    return abi::__cxa_demangle(abi::__cxa_current_exception_type()->name(), 0, 0, &status);
}

typedef boost::wave::cpplexer::lex_token<> token_type;
typedef boost::wave::cpplexer::lex_iterator<token_type> lex_iterator_type;

using context_type = boost::wave::context<std::string::iterator, lex_iterator_type, adjusted_input_policy, wave_hooks<token_type>>;
using iterator_type = boost::wave::pp_iterator<boost::wave::context<std::string::iterator, lex_iterator_type, adjusted_input_policy, wave_hooks<token_type>>>;

std::string _preprocess_text(std::string text, const char* p_filename, const std::vector<std::string> MACROS, message_callback on_message) {
    boost::wave::util::file_position_type current_position;

    try {
        std::string hashbang;
        if(std::equal(HASHBANG_PREFIX.begin(), HASHBANG_PREFIX.end(), text.begin())) {
            hashbang = text.substr(0,text.find("\n"));
            text = text.substr(text.find("\n")); // Leaves line numbers unchanged
        }

        apply_input_adjustment(text);

        context_type ctx(text.begin(), text.end(), p_filename, wave_hooks<token_type>(true, true, on_message, &current_position));

        // Configure features
        #define ENABLE(f) ctx.set_language(boost::wave::enable_##f(ctx.get_language()))
        #define DISABLE(f) ctx.set_language(boost::wave::enable_##f(ctx.get_language(), false))
        
        ENABLE(preserve_comments); // Let minifier deal with comments
        ENABLE(no_newline_at_end_of_file);
        ENABLE(no_character_validation); // '...' is just a string like "..."
        ENABLE(variadics);
        ENABLE(va_opt);
        ENABLE(has_include);
        
        DISABLE(emit_line_directives);
        DISABLE(convert_trigraphs); // Conflicts with ??=
        DISABLE(emit_pragma_directives);
        DISABLE(insert_whitespace); // Can break regexes like /\n\u{10ffff}\n/gu

        ctx.add_sysinclude_path("."); // #include <...> searches relative to pwd first
        
        // Remove unneeded macros
        #define UNDEFINE(f) ctx.remove_macro_definition(std::string(#f),true)

        // __BASE_FILE__
        // __DATE__
        // __TIME__
        // __STDC__ (required)
        // __cplusplus (required)

        UNDEFINE(__SPIRIT_PP_VERSION_STR__);
        UNDEFINE(__SPIRIT_PP_VERSION__);
        UNDEFINE(__SPIRIT_PP__);
        UNDEFINE(__WAVE_CONFIG__);
        UNDEFINE(__WAVE_VERSION_STR__);
        UNDEFINE(__WAVE_VERSION__);
        UNDEFINE(__WAVE__);

        // Add custom macros
        for(std::string macro : MACROS) {
            ctx.add_macro_definition(macro, false);
        }

        iterator_type first = ctx.begin(), last = ctx.end();
        std::stringstream out_stream;
        while (first != last) {
            current_position = (*first).get_position();
            out_stream << (*first).get_value();
            ++first;
        }
        
        std::string result = out_stream.str();
        apply_output_adjustment(result);
        
        return hashbang + result;
    }
    catch (boost::wave::cpp_exception const& e) {
        on_message(MessageType::EXCEPTION, e.file_name(), e.line_no(), e.description());
    }
    catch (std::exception const& e) {
        on_message(MessageType::EXCEPTION, current_position.get_file().c_str(), current_position.get_line(), e.what());
    }
    catch (...) {
        on_message(MessageType::EXCEPTION, current_position.get_file().c_str(), current_position.get_line(), std::string("error: unexpected exception caught (") + get_current_exception_name() + ")");
    }
    return "";
}

#include <algorithm>
namespace wave {
    rust::String preprocess_text(rust::String text, rust::String filename, const rust::Vec<rust::String> MACROS) {
        message_callback on_message = [](const MessageType TYPE, const std::string FILENAME, const i32 LINE, const std::string MESSAGE) {
           callback((i32)TYPE,FILENAME,LINE,MESSAGE);
        };
        
        std::vector<std::string> stdv;
        stdv.reserve(MACROS.size());
        std::transform(MACROS.begin(), MACROS.end(), std::back_inserter(stdv),[](const rust::String& str) { return std::string(str); });

        
        return _preprocess_text(std::string(text), filename.c_str(), stdv, on_message);
    }
}