#include "PluginProcessor.h"

#include <algorithm>

namespace {
struct Preset { const char* name; float morph, rate, delay, reverb; };
constexpr Preset presets[] = {
    { "Init", 0.5f, 0.0f, 0.0f, 0.0f },
    { "Glass Pressure", 0.82f, 0.17f, 0.18f, 0.31f },
    { "Low Embers", 0.28f, 0.06f, 0.24f, 0.48f },
};
constexpr const char* ids[] = { "base_morph", "modulation_hz", "delay_mix", "reverb_mix" };
}

juce::AudioProcessorValueTreeState::ParameterLayout ObeliskProcessor::makeParameters() {
    juce::AudioProcessorValueTreeState::ParameterLayout layout;
    const auto add = [&layout](const char* id, const char* name, float maximum, float initial) {
        layout.add(std::make_unique<juce::AudioParameterFloat>(
            juce::ParameterID { id, 1 }, name, juce::NormalisableRange<float>(0.0f, maximum), initial));
    };
    add(ids[0], "Morph", 1.0f, 0.5f);
    add(ids[1], "Modulation Hz", 20.0f, 0.0f);
    add(ids[2], "Delay Mix", 1.0f, 0.0f);
    add(ids[3], "Reverb Mix", 1.0f, 0.0f);
    return layout;
}

ObeliskProcessor::ObeliskProcessor()
    : AudioProcessor(BusesProperties().withOutput("Stereo", juce::AudioChannelSet::stereo(), true)),
      state(*this, nullptr, "ObeliskState", makeParameters()) {}

ObeliskProcessor::~ObeliskProcessor() { releaseResources(); }

void ObeliskProcessor::prepareToPlay(double sampleRate, int samplesPerBlock) {
    releaseResources();
    interleaved.resize(static_cast<size_t>(std::max(samplesPerBlock, 1)) * 2);
    engine = obelisk_m0_create(sampleRate, 16);
}

void ObeliskProcessor::releaseResources() {
    obelisk_m0_destroy(engine);
    engine = nullptr;
}

bool ObeliskProcessor::isBusesLayoutSupported(const BusesLayout& layout) const {
    return layout.getMainInputChannelSet().isDisabled()
        && layout.getMainOutputChannelSet() == juce::AudioChannelSet::stereo();
}

void ObeliskProcessor::render(juce::AudioBuffer<float>& buffer, int start, int count) {
    while (count > 0) {
        const int frames = std::min(count, static_cast<int>(interleaved.size() / 2));
        if (!obelisk_m0_process_stereo(engine, interleaved.data(), static_cast<size_t>(frames)))
            return;
        auto* left = buffer.getWritePointer(0, start);
        auto* right = buffer.getWritePointer(1, start);
        for (int i = 0; i < frames; ++i) {
            left[i] = interleaved[static_cast<size_t>(i) * 2];
            right[i] = interleaved[static_cast<size_t>(i) * 2 + 1];
        }
        start += frames;
        count -= frames;
    }
}

void ObeliskProcessor::processBlock(juce::AudioBuffer<float>& buffer, juce::MidiBuffer& midi) {
    juce::ScopedNoDenormals noDenormals;
    buffer.clear();
    if (engine == nullptr)
        return;

    const auto value = [this](const char* id) { return state.getRawParameterValue(id)->load(); };
    obelisk_m0_set_controls(engine, value(ids[0]), value(ids[1]), value(ids[2]), value(ids[3]));

    const int total = buffer.getNumSamples();
    int cursor = 0;
    for (const auto metadata : midi) {
        const int offset = juce::jlimit(0, total, metadata.samplePosition);
        if (offset > cursor) {
            render(buffer, cursor, offset - cursor);
            cursor = offset;
        }
        const auto message = metadata.getMessage();
        if (message.getRawDataSize() >= 2 && message.getRawDataSize() <= 3) {
            const auto* bytes = message.getRawData();
            const auto status = bytes[0];
            if (status >= 0x80 && status < 0xf0)
                obelisk_m0_send_midi3(engine, status, bytes[1],
                                      message.getRawDataSize() == 3 ? bytes[2] : 0);
        }
    }
    if (cursor < total)
        render(buffer, cursor, total - cursor);
    midi.clear();
}

juce::AudioProcessorEditor* ObeliskProcessor::createEditor() {
    return new juce::GenericAudioProcessorEditor(*this);
}

const juce::String ObeliskProcessor::getProgramName(int index) {
    return juce::isPositiveAndBelow(index, 3) ? presets[index].name : juce::String();
}

void ObeliskProcessor::setCurrentProgram(int index) {
    if (!juce::isPositiveAndBelow(index, 3))
        return;
    const auto& p = presets[index];
    const float values[] = { p.morph, p.rate, p.delay, p.reverb };
    for (int i = 0; i < 4; ++i)
        if (auto* parameter = state.getParameter(ids[i]))
            parameter->setValueNotifyingHost(parameter->convertTo0to1(values[i]));
    program.store(index);
}

void ObeliskProcessor::getStateInformation(juce::MemoryBlock& destination) {
    auto tree = state.copyState();
    tree.setProperty("program", program.load(), nullptr);
    if (auto xml = tree.createXml())
        copyXmlToBinary(*xml, destination);
}

void ObeliskProcessor::setStateInformation(const void* data, int size) {
    if (auto xml = getXmlFromBinary(data, size)) {
        if (!xml->hasTagName(state.state.getType().toString()))
            return;
        auto tree = juce::ValueTree::fromXml(*xml);
        if (!tree.isValid())
            return;
        program.store(juce::jlimit(0, 2, static_cast<int>(tree.getProperty("program", 0))));
        tree.removeProperty("program", nullptr);
        state.replaceState(tree);
    }
}

juce::AudioProcessor* JUCE_CALLTYPE createPluginFilter() { return new ObeliskProcessor(); }
