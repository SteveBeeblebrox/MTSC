/*=============================================================================
    Boost.Wave: A Standard compliant C++ preprocessor library

    Definition of the predefined macros

    http://www.boost.org/

    Copyright (c) 2001-2012 Hartmut Kaiser. Distributed under the Boost
    Software License, Version 1.0. (See accompanying file
    LICENSE_1_0.txt or copy at http://www.boost.org/LICENSE_1_0.txt)
=============================================================================*/

#if !defined(BOOST_CPP_MACROMAP_PREDEF_HPP_HK041119)
#define BOOST_CPP_MACROMAP_PREDEF_HPP_HK041119

#include <cstdio>
#include <boost/assert.hpp>
#include <boost/format.hpp>

#include <boost/wave/wave_config.hpp>
#include <boost/wave/wave_config_constant.hpp>
#include <boost/wave/token_ids.hpp>
#include <boost/wave/util/time_conversion_helper.hpp> // time_conversion_helper

// this must occur after all of the includes and before any code appears
#ifdef BOOST_HAS_ABI_HEADERS
#include BOOST_ABI_PREFIX
#endif

///////////////////////////////////////////////////////////////////////////////
//
// This file contains the definition of functions needed for the management
// of static and dynamic predefined macros, such as __DATE__, __TIME__ etc.
//
// Note: __FILE__, __LINE__ and __INCLUDE_LEVEL__ are handled in the file
//       cpp_macromap.hpp.
//
///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////
namespace boost {
namespace wave {
namespace util {

    ///////////////////////////////////////////////////////////////////////////
    class predefined_macros
    {
        typedef BOOST_WAVE_STRINGTYPE string_type;

    public:
        // list of static predefined macros
        struct static_macros {
            char const *name;
            boost::wave::token_id token_id;
            char const *value;
        };

        // list of dynamic predefined macros
        struct dynamic_macros {
            char const *name;
            boost::wave::token_id token_id;
            string_type (predefined_macros:: *generator)() const;
        };

    private:
        boost::wave::util::time_conversion_helper compilation_time_;
        string_type datestr_;     // __DATE__
        string_type timestr_;     // __TIME__
        string_type version_;     // __SPIRIT_PP_VERSION__/__WAVE_VERSION__
        string_type versionstr_;  // __SPIRIT_PP_VERSION_STR__/__WAVE_VERSION_STR__

        static string_type strconv(std::string const & ss)
        {
            return string_type(ss.c_str());
        }

    protected:
        void reset_datestr()
        {
            static const char *const monthnames[] = {
                "Jan", "Feb", "Mar", "Apr", "May", "Jun",
                "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
            };

            // for some systems sprintf, time_t etc. is in namespace std
            using namespace std;

            time_t tt = time(0);
            struct tm *tb = 0;

            if (tt != (time_t)-1) {
                tb = localtime (&tt);
                datestr_ = strconv((boost::format("\"%s %2d %4d\"")
                                    % monthnames[tb->tm_mon]
                                    % tb->tm_mday
                                    % (tb->tm_year + 1900)).str());
            }
            else {
                datestr_ = "\"??? ?? ????\"";
            }
        }

        void reset_timestr()
        {
            // for some systems sprintf, time_t etc. is in namespace std
            using namespace std;

            time_t tt = time(0);
            struct tm *tb = 0;

            if (tt != (time_t)-1) {
                tb = localtime (&tt);
                timestr_ = strconv((boost::format("\"%02d:%02d:%02d\"")
                                    % tb->tm_hour
                                    % tb->tm_min
                                    % tb->tm_sec).str());
            }
            else {
                timestr_ = "\"??:??:??\"";
            }
        }

        void reset_version()
        {
            // for some systems sprintf, time_t etc. is in namespace std
            using namespace std;

            // calculate the number of days since Dec 13 2001
            // (the day the Wave project was started)
            tm first_day;

            using namespace std;    // for some systems memset is in namespace std
            memset (&first_day, 0, sizeof(tm));
            first_day.tm_mon = 11;           // Dec
            first_day.tm_mday = 13;          // 13
            first_day.tm_year = 101;         // 2001

            long seconds = long(difftime(compilation_time_.get_time(), mktime(&first_day)));

            version_ = strconv((boost::format("0x%02d%1d%1d%04ld")
                                % BOOST_WAVE_VERSION_MAJOR
                                % BOOST_WAVE_VERSION_MINOR
                                % BOOST_WAVE_VERSION_SUBMINOR
                                % (seconds/(3600*24))).str());
        }

        void reset_versionstr()
        {
            // for some systems sprintf, time_t etc. is in namespace std
            using namespace std;

            // calculate the number of days since Dec 13 2001
            // (the day the Wave project was started)
            tm first_day;

            memset (&first_day, 0, sizeof(tm));
            first_day.tm_mon = 11;           // Dec
            first_day.tm_mday = 13;          // 13
            first_day.tm_year = 101;         // 2001

            long seconds = long(difftime(compilation_time_.get_time(), mktime(&first_day)));

            versionstr_ = strconv((boost::format("\"%d.%d.%d.%ld [%s/%s]\"")
                                   % BOOST_WAVE_VERSION_MAJOR
                                   % BOOST_WAVE_VERSION_MINOR
                                   % BOOST_WAVE_VERSION_SUBMINOR
                                   % (seconds/(3600*24))
                                   % BOOST_PLATFORM
                                   % BOOST_COMPILER).str());
        }

        // dynamic predefined macros
        string_type get_date() const { return datestr_; }     // __DATE__
        string_type get_time() const { return timestr_; }     // __TIME__

        // __SPIRIT_PP__/__WAVE__
        string_type get_version() const
        {
            return strconv((boost::format("0x%02d%1d%1d")
                            % BOOST_WAVE_VERSION_MAJOR
                            % BOOST_WAVE_VERSION_MINOR
                            % BOOST_WAVE_VERSION_SUBMINOR).str());
        }

        // __WAVE_CONFIG__
        string_type get_config() const
        {
            return strconv((boost::format("0x%08x") % BOOST_WAVE_CONFIG).str());
        }

    public:
        predefined_macros()
          : compilation_time_(__DATE__ " " __TIME__)
        {
            reset();
            reset_version();
            reset_versionstr();
        }

        void reset()
        {
            reset_datestr();
            reset_timestr();
        }

        // __SPIRIT_PP_VERSION__/__WAVE_VERSION__
        string_type get_fullversion() const { return version_; }

        // __SPIRIT_PP_VERSION_STR__/__WAVE_VERSION_STR__
        string_type get_versionstr() const { return versionstr_; }

        // C++ mode
        static_macros const& static_data_cpp(std::size_t i) const
        {
        static static_macros data[] = {
                // Patch { "__STDC__", T_INTLIT, "1" },
                // Patch { "__cplusplus", T_INTLIT, "199711L" },
                /*Start Patch*/ { "__NEWLINE__", T_NEWLINE, "\n"}, /*End Patch*/
                { 0, T_EOF, 0 }
            };
            BOOST_ASSERT(i < sizeof(data)/sizeof(data[0]));
            return data[i];
        }

#if BOOST_WAVE_SUPPORT_CPP0X != 0
        // C++11 mode
        static_macros const& static_data_cpp0x(std::size_t i) const
        {
        static static_macros data[] = {
                // Patch { "__STDC__", T_INTLIT, "1" },
                // Patch { "__cplusplus", T_INTLIT, "201103L" },
                // Patch { "__STDC_VERSION__", T_INTLIT, "199901L" },
                // Patch { "__STDC_HOSTED__", T_INTLIT, "0" },
                // Patch { "__WAVE_HAS_VARIADICS__", T_INTLIT, "1" },
                /*Start Patch*/ { "__NEWLINE__", T_NEWLINE, "\n"}, /*End Patch*/
                { 0, T_EOF, 0 }
            };
            BOOST_ASSERT(i < sizeof(data)/sizeof(data[0]));
            return data[i];
        }
#endif

#if BOOST_WAVE_SUPPORT_CPP2A != 0
        // C++20 mode
        static_macros const& static_data_cpp2a(std::size_t i) const
        {
        static static_macros data[] = {
                // Patch { "__STDC__", T_INTLIT, "1" },
                // Patch { "__cplusplus", T_INTLIT, "202002L" },
                // Patch { "__STDC_VERSION__", T_INTLIT, "199901L" },
                // Patch { "__STDC_HOSTED__", T_INTLIT, "0" },
                // Patch { "__WAVE_HAS_VARIADICS__", T_INTLIT, "1" },
                /*Start Patch*/ { "__NEWLINE__", T_NEWLINE, "\n"}, /*End Patch*/
                { 0, T_EOF, 0 }
            };
            BOOST_ASSERT(i < sizeof(data)/sizeof(data[0]));
            return data[i];
        }
#endif

#if BOOST_WAVE_SUPPORT_VARIADICS_PLACEMARKERS != 0
        // C99 mode
        static_macros const& static_data_c99(std::size_t i) const
        {
        static static_macros data[] = {
                // Patch { "__STDC__", T_INTLIT, "1" },
                // Patch { "__STDC_VERSION__", T_INTLIT, "199901L" },
                // Patch { "__STDC_HOSTED__", T_INTLIT, "0" },
                // Patch { "__WAVE_HAS_VARIADICS__", T_INTLIT, "1" },
                /*Start Patch*/ { "__NEWLINE__", T_NEWLINE, "\n"}, /*End Patch*/
                { 0, T_EOF, 0 }
            };
            BOOST_ASSERT(i < sizeof(data)/sizeof(data[0]));
            return data[i];
        }
#endif

        dynamic_macros const& dynamic_data(std::size_t i) const
        {
        static dynamic_macros data[] = {
                { "__DATE__", T_STRINGLIT, &predefined_macros::get_date },
                { "__TIME__", T_STRINGLIT, &predefined_macros::get_time },
                { "__SPIRIT_PP__", T_INTLIT, &predefined_macros::get_version },
                { "__SPIRIT_PP_VERSION__", T_INTLIT, &predefined_macros::get_fullversion },
                { "__SPIRIT_PP_VERSION_STR__", T_STRINGLIT, &predefined_macros::get_versionstr },
                { "__WAVE__", T_INTLIT, &predefined_macros::get_version },
                { "__WAVE_VERSION__", T_INTLIT, &predefined_macros::get_fullversion },
                { "__WAVE_VERSION_STR__", T_STRINGLIT, &predefined_macros::get_versionstr },
                { "__WAVE_CONFIG__", T_INTLIT, &predefined_macros::get_config },
                { 0, T_EOF, 0 }
            };
            BOOST_ASSERT(i < sizeof(data)/sizeof(data[0]));
            return data[i];
        }
    };   // predefined_macros

///////////////////////////////////////////////////////////////////////////////
}   // namespace util
}   // namespace wave
}   // namespace boost

// the suffix header occurs after all of the code
#ifdef BOOST_HAS_ABI_HEADERS
#include BOOST_ABI_SUFFIX
#endif

#endif // !defined(BOOST_CPP_MACROMAP_PREDEF_HPP_HK041119)
