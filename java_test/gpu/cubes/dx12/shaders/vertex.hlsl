struct VSInput {
    float3 pos : POSITION;
    float3 color : COLOR;
};
struct PSInput {
    float4 pos : SV_POSITION;
    float3 color : COLOR;
};
cbuffer Transform : register(b0) {
    float4x4 MVP;
};
PSInput VSMain(VSInput input) {
    PSInput output;
    output.pos = mul(MVP, float4(input.pos, 1));
    output.color = input.color;
    return output;
}
