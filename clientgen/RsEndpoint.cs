using System;
using System.Collections.Generic;
using System.Linq;
using RestApisGen;
using System.Text.RegularExpressions;

namespace Tweetust.ClientGen
{
    class RsEndpoint
    {
        public RsEndpoint(ApiEndpoint source)
        {
            this.Name = source.Name;
            this.Method = source.Request;
            this.ReturnType = source.ReturnType; //TODO
            this.Uri = source.Uri;
            this.ReservedParameter = source.ReservedName;
            this.RequiredParameters = ToRsParameterArray(source.Params.Where(x => x.Kind == "required"));
            var either = source.Params.Where(x => x.Kind == "any one is required").Distinct(comparer).ToArray();
            if (this.RequiredParameters.Count == 0 && either.Count() == 1)
            {
                this.RequiredParameters = ToRsParameterArray(either);
                this.OptionalParameters = ToRsParameterArray(
                    source.Params.Where(x => x.Kind != "required" && x.Kind != "any one is required"));
            }
            else
            {
                this.OptionalParameters = ToRsParameterArray(source.Params.Where(x => x.Kind != "required"));
            }
        }

        public string Name;
        public string Method;
        public string ReturnType;
        public string Uri;
        public string ReservedParameter;
        public IReadOnlyList<RsParameter> RequiredParameters;
        public IReadOnlyList<RsParameter> OptionalParameters;

        private class PrmComparer : IEqualityComparer<Parameter>
        {
            public bool Equals(Parameter x, Parameter y)
            {
                if (x == null) return y == null;
                if (y == null) return false;
                return x.RealName == y.RealName;
            }

            public int GetHashCode(Parameter obj)
            {
                return obj.RealName.GetHashCode();
            }
        }
        private static PrmComparer comparer = new PrmComparer();
        private static RsParameter[] ToRsParameterArray(IEnumerable<Parameter> source)
        {
            return source.Distinct(comparer).Select(x => new RsParameter(x)).ToArray();
        }
    }

    struct RsParameter
    {
        public RsParameter(Parameter source)
        {
            this.Name = source.RealName;
            var t = source.Type;
            var m = Regex.Match(t, "^IEnumerable<(.+)>$");
            if (m.Success)
            {
                this.Type = new VecType(ConvertType(m.Groups[1].Value).ToString());
                return;
            }
            this.Type = ConvertType(t);
        }

        private static IRsType ConvertType(string t)
        {
            switch (t)
            {
                case "string":
                    return new StringType();
                case "int":
                    return new RawType("i32");
                case "long":
                    return new RawType("i64");
                case "double":
                    return new RawType("f64");
                default:
                    return new RawType(t);
            }
        }

        public string Name;
        public IRsType Type;
    }

    interface IRsType
    {
        T Match<T>(Func<RawType, T> rawType, Func<StringType, T> stringType, Func<VecType, T> vecType);
    }

    class RawType : IRsType
    {
        public RawType(string type)
        {
            this.Type = type;
        }

        public readonly string Type;

        public T Match<T>(Func<RawType, T> rawType, Func<StringType, T> stringType, Func<VecType, T> vecType)
        {
            return rawType(this);
        }

        public override string ToString()
        {
            return this.Type;
        }
    }

    class StringType : IRsType
    {
        public T Match<T>(Func<RawType, T> rawType, Func<StringType, T> stringType, Func<VecType, T> vecType)
        {
            return stringType(this);
        }

        public override string ToString()
        {
            return "String";
        }
    }

    class VecType : IRsType
    {
        public VecType(string type)
        {
            this.Type = type;
        }

        public readonly string Type;

        public T Match<T>(Func<RawType, T> rawType, Func<StringType, T> stringType, Func<VecType, T> vecType)
        {
            return vecType(this);
        }

        public override string ToString()
        {
            return string.Concat("Vec<", this.Type, ">");
        }
    }
}
