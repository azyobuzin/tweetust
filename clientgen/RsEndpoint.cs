using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using RestApisGen;

namespace Tweetust.ClientGen
{
    class RsEndpoint
    {
        public RsEndpoint(ApiEndpoint source)
        {
            this.Source = source;
            this.Name = source.Name;
            this.Method = source.Request;
            this.ReturnType = CreateReturnType(source);
            this.Uri = source.Uri;
            this.ReservedParameter = source.ReservedName;

            var areAllOptional = source.AnyOneGroups.Any(x => !x.Any());
            var isEitherRequired = false;
            if (!areAllOptional && source.AnyOneGroups.Count > 0)
            {
                var eithered = Extensions.Combinate(source.AnyOneGroups).ToArray();
                var prmNames = eithered[0].SelectMany(x => x.Select(y => y.RealName)).ToArray();
                isEitherRequired = eithered.All(x => x.SelectMany(y => y.Select(z => z.RealName)).SequenceEqual(prmNames));
            }

            var requiredParameters = new List<RsParameter>();
            var optionalParameters = new List<RsParameter>();

            foreach (var param in source.Params.Distinct(comparer))
            {
                (param.Kind == "required" || (param.Kind == "any one is required" && isEitherRequired)
                    ? requiredParameters : optionalParameters).Add(new RsParameter(param));
            }

            this.RequiredParameters = requiredParameters;
            this.OptionalParameters = optionalParameters;
        }

        public ApiEndpoint Source; //TODO: remove
        public string Name;
        public string Method;
        public RsType ReturnType;
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
        private static readonly PrmComparer comparer = new PrmComparer();

        private static RsType CreateReturnType(ApiEndpoint endpoint)
        {
            var t = endpoint.ReturnType;

            if (t == "StringResponse")
                return new RawType(endpoint.Name + "Response");

            if (t.EndsWith("Response"))
                t = t.Substring(0, t.Length - 8);

            switch (t)
            {
                case "Status":
                    t = "Tweet";
                    break;
                case "Embed":
                    t = "OEmbed";
                    break;
                case "SearchResult":
                    t = "SearchResponse";
                    break;
                case "Configurations":
                    t = "Configuration";
                    break;
                case "TrendLocation":
                    t = "TrendPlace";
                    break;
                //TODO: support
                case "Setting":
                case "GeoResult":
                case "SearchQuery":
                case "ProfileBannerSizes":
                case "Category":
                    return null;
            }

            switch (endpoint.Type)
            {
                case ApiType.Void:
                    return new UnitType();
                case ApiType.Normal:
                    return RsType.FromString(t);
                case ApiType.Listed:
                    return new VecType(RsType.FromString(t));
                case ApiType.Cursored:
                    if (t == "long") t = "Id";
                    return new RawType("Cursor" + t + "s");
                case ApiType.Dictionary:
                    return null;
                default:
                    throw new ArgumentException("apiType");
            }
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
                this.Type = new VecType(RsType.FromString(m.Groups[1].Value));
                return;
            }
            this.Type = RsType.FromString(t);
        }

        public string Name;
        public RsType Type;
    }

    abstract class RsType
    {
        public abstract T Match<T>(Func<RawType, T> rawType, Func<StringType, T> stringType, Func<VecType, T> vecType, Func<UnitType, T> unitType);

        public static RsType FromString(string t)
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
    }

    class RawType : RsType
    {
        public RawType(string type)
        {
            this.Type = type;
        }

        public readonly string Type;

        public override T Match<T>(Func<RawType, T> rawType, Func<StringType, T> stringType, Func<VecType, T> vecType, Func<UnitType, T> unitType)
        {
            return rawType(this);
        }

        public override string ToString()
        {
            return this.Type;
        }
    }

    class StringType : RsType
    {
        public override T Match<T>(Func<RawType, T> rawType, Func<StringType, T> stringType, Func<VecType, T> vecType, Func<UnitType, T> unitType)
        {
            return stringType(this);
        }

        public override string ToString()
        {
            return "String";
        }
    }

    class VecType : RsType
    {
        public VecType(RsType type)
        {
            this.Type = type;
        }

        public readonly RsType Type;

        public override T Match<T>(Func<RawType, T> rawType, Func<StringType, T> stringType, Func<VecType, T> vecType, Func<UnitType, T> unitType)
        {
            return vecType(this);
        }

        public override string ToString()
        {
            return string.Concat("Vec<", this.Type.ToString(), ">");
        }
    }

    class UnitType : RsType
    {
        public override T Match<T>(Func<RawType, T> rawType, Func<StringType, T> stringType, Func<VecType, T> vecType, Func<UnitType, T> unitType)
        {
            return unitType(this);
        }

        public override string ToString()
        {
            return "()";
        }
    }
}
