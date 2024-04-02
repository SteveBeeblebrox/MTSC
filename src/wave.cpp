#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <sstream>
#include <cstdlib>
#include <stack>
#include <filesystem>
#include <algorithm>

// Static wave configuration
#define BOOST_WAVE_SUPPORT_CPP1Z 1
#define BOOST_WAVE_SUPPORT_MS_EXTENSIONS 0
#define BOOST_WAVE_PRAGMA_KEYWORD "mtsc"
#define BOOST_WAVE_SUPPORT_PRAGMA_MESSAGE 0

#include <boost/wave.hpp>
#include <boost/wave/cpplexer/cpp_lex_token.hpp>
#include <boost/wave/cpplexer/cpp_lex_iterator.hpp>
#include <boost/regex.hpp>
#include <boost/lexical_cast.hpp>
#include <boost/format.hpp>

#define UFFFF "\uffff"
#define UFFFE "\ufffe"

#include "wave.hpp"
using namespace wave;

typedef std::function<void(const MessageType TYPE, std::string filename, const i32 LINE, const std::string MESSAGE)> message_callback;

typedef boost::wave::cpplexer::lex_token<> token_type;
typedef boost::wave::cpplexer::lex_iterator<token_type> lex_iterator_type;
typedef boost::wave::util::file_position_type position_type;

struct adjusted_input_policy;

template<typename TokenT>
class wave_hooks;

using context_type = boost::wave::context<std::string::iterator, lex_iterator_type, adjusted_input_policy, wave_hooks<token_type>>;
using iterator_type = boost::wave::pp_iterator<context_type>;

const std::string HASHBANG_PREFIX = "#!";

template<typename T>
inline std::string as_hex_literal(T t) {
    return (boost::format("%1$#x") % ((unsigned int)t)).str();
}

template<typename IteratorT>
inline std::string as_unescaped_string(IteratorT it, IteratorT const& end) {    
    std::string result;
    while(it != end) {
        switch (boost::wave::token_id(*it)) {
            case boost::wave::T_STRINGLIT: {
                    std::string val (boost::wave::util::impl::unescape_lit((*it).get_value()).c_str());
                    val.erase(val.size()-1);
                    val.erase(0, 1);
                    result += val;
                }
                break;
            default:
                break;
        }
        it++;
    }
    return result;
}

template<typename ContainerT>
inline std::string as_unescaped_string(ContainerT const &token_sequence) {
    return as_unescaped_string(token_sequence.begin(), token_sequence.end());
}

template<typename ContextT>
struct reset_language_support {
    ContextT& ctx_;
    boost::wave::language_support lang_;

    reset_language_support(ContextT& ctx) : ctx_(ctx), lang_(ctx.get_language()) {
        ctx.set_language(boost::wave::enable_single_line(lang_), false);
    }
    ~reset_language_support() {
        ctx_.set_language(lang_, false);
    }
};

template<typename TokenT>
class wave_hooks : public boost::wave::context_policies::eat_whitespace<TokenT>
{
    typedef boost::wave::context_policies::eat_whitespace<TokenT> base_type;
    private:
        const bool PRESERVE_WHITESPACE;       // enable whitespace preservation
        const bool PRESERVE_BOL_WHITESPACE;   // enable beginning of line whitespace preservation
        message_callback on_message;
        position_type& current_position;
        iterator_type*& iter;                 // reference to a pointer to an iterator

    public:
        wave_hooks(const bool PRESERVE_WHITESPACE, const bool PRESERVE_BOL_WHITESPACE, message_callback on_message, position_type& current_position, iterator_type*& iter) : PRESERVE_WHITESPACE(PRESERVE_WHITESPACE), PRESERVE_BOL_WHITESPACE(PRESERVE_BOL_WHITESPACE), on_message(on_message), current_position(current_position), iter(iter) {}

        template<typename ContextT>
        bool may_skip_whitespace(ContextT const &ctx, TokenT &token, bool &skipped_newline) {
            return this->base_type::may_skip_whitespace(
                    ctx, token, need_preserve_comments(ctx.get_language()),
                    PRESERVE_BOL_WHITESPACE, skipped_newline) ?
                !PRESERVE_WHITESPACE : false;
        }

        template<typename ContextT, typename ContainerT>
        bool interpret_pragma(ContextT& ctx, ContainerT &pending, TokenT const& option, ContainerT const& values, TokenT const& act_token) {
            if(option.get_value() == "eval") {
                try {
                    std::string source = as_unescaped_string(values);
                    reset_language_support<ContextT> lang(ctx);

                    std::cerr<<"Eval: "<<source<<std::endl;
                    // ctx.push_iteration_context(ctx.get_main_pos(),iter->get_functor().iter_ctx);

                    // ContainerT pragma;
                    // iterator_type it = ctx.begin(source.begin(), source.end());
                    // iterator_type end = ctx.end();
                    

                    // pending.push_back(*it);
                    // pending.push_back(*++it);

                    // while(it != ctx.end() && boost::wave::token_id(*it) != boost::wave::T_EOF) {
                    //     std::cerr<<"::"<<it->get_value()<<"::"<<boost::wave::get_token_name(boost::wave::token_id(*it))<<std::endl;
                    //     // pragma.push_back(*it);
                    //     ++it;
                    // }

                    // iter->get_functor().iter_ctx = ctx.pop_iteration_context();

                    // pending.splice(pending.begin(), pragma);
                    
                    return true;
                } catch(boost::wave::cpp_exception const& e) {
                    std::cerr<<e.description()<<std::endl;
                }
                catch(boost::wave::cpplexer::lexing_exception const& e) {
                    std::cerr<<e.description()<<std::endl;
                }
                catch(std::exception const& e) {
                    std::cerr<<e.what()<<std::endl;
                } catch(...) {
                    return false;
                }
            } else if(option.get_value() == "line") {
                typedef typename ContainerT::const_iterator value_iterator_type;
                try {
                    int value;
                    int line = iter->get_functor().iter_ctx->first->get_position().get_line();
                    value_iterator_type value_iter = values.begin();
                    auto get_int_value = [&](value_iterator_type& value_iter, int& out) {
                        if(boost::wave::token_id(*value_iter) == boost::wave::T_PP_NUMBER) {
                            out = boost::lexical_cast<int>(value_iter->get_value().c_str());
                            return ++value_iter == values.end();
                        } else {
                            return false;
                        }
                    };
                    switch(boost::wave::token_id(*value_iter)) {
                        case boost::wave::T_PLUS: {
                            if(!get_int_value(++value_iter, value)) {
                                return false;
                            }

                            line += value;

                            break;
                        }
                        case boost::wave::T_MINUS: {
                            if(!get_int_value(++value_iter, value)) {
                                return false;
                            }

                            line -= value;

                            break;
                        }
                        case boost::wave::T_PP_NUMBER: {
                            if(!get_int_value(value_iter, value)) {
                                return false;
                            }

                            line = value;

                            break;
                        }
                        default: {
                            return false;
                        }
                    }

                    position_type npos(current_position);
                    npos.set_line(line);
                    iter->get_functor().iter_ctx->first.set_position(npos);

                    return true;
                } catch(...) {
                    return false;
                }
            }

            return false;
        }

        template<typename ContextT, typename ContainerT>
        bool found_warning_directive(ContextT const& ctx, ContainerT const& message) {
            on_message(MessageType::WARNING, current_position.get_file().c_str(), current_position.get_line(), boost::wave::util::impl::as_string(message).c_str());
            return true;
        }

        template<typename ContextT, typename ContainerT>
        bool found_error_directive(ContextT const& ctx, ContainerT const& message) {
            on_message(MessageType::ERROR, current_position.get_file().c_str(), current_position.get_line(), boost::wave::util::impl::as_string(message).c_str());
            return true;
        }

        template<typename ContextT, typename ContainerT>
        bool found_unknown_directive(ContextT& ctx, ContainerT const& line, ContainerT& pending) {
            typedef typename ContainerT::const_iterator iterator_type;
            iterator_type it = line.begin();
            boost::wave::token_id id = boost::wave::util::impl::skip_whitespace(it, line.end());

            if(id != boost::wave::T_IDENTIFIER) {
                return false;
            }

            if((*it).get_value() == "embed") {
                typename ContextT::position_type pos = it->get_position();
                size_t column = pos.get_column();
                std::string value = as_unescaped_string(++it,line.end());
                std::string dir,path;
                if(!this->locate_include_file(ctx,value,false,NULL,dir,path)) {
                    return false;
                }

                std::ifstream stream(path,std::ios::in | std::ios::binary);
                stream.unsetf(std::ios_base::skipws);
            
                std::istream_iterator<unsigned char> start(stream);
                std::istream_iterator<unsigned char> end;
                while(start != end) {
                    pos.set_column(column);
                    std::string lit = as_hex_literal(*(start++));
                    pending.push_back(TokenT(boost::wave::T_HEXAINT, lit.c_str(), pos));
                    column += (size_t) lit.length();
                    if(start != end) {
                        pos.set_column(column);
                        pending.push_back(TokenT(boost::wave::T_COMMA, ",", pos));
                        column++;
                    }
                }

                return true;
            }

            return false;
        }
};

// See adjustments.ts
// \u{fffe}0;       Scope: apply_input_adjustment (marks escaped newlines)
// \u{ffff}17;      Scope: preprocess_text (escape normal hashtags like those in private properties)
// \u{ffff}91;      Scope: preprocess_text (mark split lines)

std::string& apply_input_adjustment(std::string &text, const bool DISCARD_HASHBANG = false) {
    auto replace = [&](const boost::regex& regex, const std::string& replacement) {
        return text = boost::regex_replace(text,regex,replacement);
    };

    if(DISCARD_HASHBANG && std::equal(HASHBANG_PREFIX.begin(), HASHBANG_PREFIX.end(), text.begin())) {
        text = text.substr(text.find("\n")); // Leaves line numbers unchanged
    }
    
    replace(boost::regex("\\\\\\r?\\n"), UFFFE "0;");                           // Unescape newlines
    /////////////////////////////////

    replace(boost::regex("^(?=\\s*?#)"), UFFFF "17;");                          // Block normal #...
    replace(boost::regex("^(\\s*?)\\/\\/\\/(?=\\s*?#)"), "$1" UFFFF "91;\n#pragma mtsc line(-2)\n");   // Split ///#...

    /////////////////////////////////
    replace(boost::regex(UFFFE "0;"), "\\\\\n");                                // Re-escape newlines

    text += "\n"; // Add an extra \n to the end; wave fails on a trailing comment

    return text;
}

std::string& apply_output_adjustment(std::string &text) {
    auto replace = [&](const boost::regex& regex, const std::string& replacement) {
        return text = boost::regex_replace(text,regex,replacement);
    };

    replace(boost::regex(UFFFF "17;"), "");                                     // Unblock normal #...
    replace(boost::regex(UFFFF "91;\\r?\\n(?=\\s*?#)"), "///");                 // Unsplit unused ///#
    replace(boost::regex(UFFFF "91;\\r?\\n"), "");                              // Discard used ///# splits

    return text;
}

struct adjusted_input_policy {
    template<typename IterContextT>
    class inner {
        public:
            template<typename PositionT>
            static void init_iterators(IterContextT &iter_ctx, PositionT const &act_pos, boost::wave::language_support language) {
                typedef typename IterContextT::iterator_type iterator_type;

                boost::filesystem::ifstream instream(iter_ctx.filename.c_str());
                if(!instream.is_open()) {
                    BOOST_WAVE_THROW_CTX(iter_ctx.ctx, boost::wave::preprocess_exception,
                        bad_include_file, iter_ctx.filename.c_str(), act_pos);
                    return;
                }
                instream.unsetf(std::ios::skipws);

                iter_ctx.instring.assign(
                    std::istreambuf_iterator<char>(instream.rdbuf()),
                    std::istreambuf_iterator<char>());

                apply_input_adjustment(iter_ctx.instring, true);

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

std::string _preprocess_text(std::string text, const char* p_filename, const std::vector<std::string> MACROS, const std::vector<std::string> INCLUDE_PATHS, message_callback on_message) {
    boost::wave::util::file_position_type current_position;

    try {
        std::string hashbang;
        if(std::equal(HASHBANG_PREFIX.begin(), HASHBANG_PREFIX.end(), text.begin())) {
            hashbang = text.substr(0,text.find("\n"));
            text = text.substr(text.find("\n")); // Leaves line numbers unchanged
        }

        apply_input_adjustment(text);

        iterator_type* iter;
        context_type ctx(text.begin(), text.end(), p_filename, wave_hooks<token_type>(true, true, on_message, current_position, iter));

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

        ctx.add_macro_definition("__MTSC_VERSION__=\"" MTSC_VERSION "\"", true);

        // Add custom macros
        for(std::string macro : MACROS) {
            ctx.add_macro_definition(macro, false);
        }

        // #include <...> prefers cli -I paths and main input folder (if known) over relative paths
        // #include "..." prefers relative paths over cli -I paths

        // Add custom include paths
        for(std::string path : INCLUDE_PATHS) {
            ctx.add_sysinclude_path(path.c_str());
        }

        if(std::string(p_filename) != "-") {
            std::string path = std::filesystem::absolute(std::filesystem::path(p_filename)).parent_path().string();
            ctx.add_sysinclude_path(path.c_str());
        }

        iterator_type first = ctx.begin(), last = ctx.end();
        std::stringstream out_stream;

        bool need_to_advance = false, finished = false;
        do {
            try {
                if(need_to_advance) {
                    ++first;
                    need_to_advance = false;
                }
                while(first != last) {
                    iter=&first;
                    current_position = (*first).get_position();
                    out_stream << (*first).get_value();
                    ++first;
                }
                finished = true;
            } catch(boost::wave::cpp_exception const &e) {
                if(boost::wave::is_recoverable(e)) {
                    need_to_advance = true;
                    on_message(MessageType::WARNING, e.file_name(), e.line_no(), e.description());
                }
                else {
                    throw;
                }
            }
            catch(boost::wave::cpplexer::lexing_exception const &e) {
                if(boost::wave::cpplexer::is_recoverable(e)) {
                    need_to_advance = true;
                    on_message(MessageType::WARNING, e.file_name(), e.line_no(), e.description());
                }
                else {
                    throw;
                }
            }
        } while(!finished);
        
        std::string result = out_stream.str();
        apply_output_adjustment(result);

        return hashbang + result;
    }
    catch(boost::wave::cpp_exception const& e) {
        on_message(MessageType::EXCEPTION, e.file_name(), e.line_no(), e.description());
    }
    catch(boost::wave::cpplexer::lexing_exception const& e) {
        on_message(MessageType::EXCEPTION, current_position.get_file().c_str(), current_position.get_line(), e.description());
    }
    catch(std::exception const& e) {
        on_message(MessageType::EXCEPTION, current_position.get_file().c_str(), current_position.get_line(), e.what());
    }
    catch(...) {
        on_message(MessageType::EXCEPTION, current_position.get_file().c_str(), current_position.get_line(), std::string("error: unexpected exception caught (") + get_current_exception_name() + ")");
    }
    return "";
}

namespace wave {
    rust::String preprocess_text(rust::String text, rust::String filename, const rust::Vec<rust::String> MACROS, const rust::Vec<rust::String> INCLUDE_PATHS) {
        message_callback on_message = [](const MessageType TYPE, const std::string FILENAME, const i32 LINE, const std::string MESSAGE) {
           callback((i32)TYPE,FILENAME,LINE,MESSAGE);
        };
        
        std::vector<std::string> macros;
        macros.reserve(MACROS.size());
        std::transform(MACROS.begin(), MACROS.end(), std::back_inserter(macros), [](const rust::String& str) { return std::string(str); });

        std::vector<std::string> paths;
        paths.reserve(INCLUDE_PATHS.size());
        std::transform(INCLUDE_PATHS.begin(), INCLUDE_PATHS.end(), std::back_inserter(paths), [](const rust::String& str) { return std::string(str); });


        return _preprocess_text(std::string(text), filename.c_str(), macros, paths, on_message);
    }
}